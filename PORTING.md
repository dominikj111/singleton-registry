# Porting singleton-registry to Other Languages

This document provides guidance for implementing a similar singleton registry in other programming languages. The goal is **native ports**, not FFI bindings—each implementation should be idiomatic to its target language.

## Core Design Decisions

### 1. Type-Keyed Storage

The registry maps **types** to **single instances**. Each language has its own mechanism:

| Language   | Type Key Mechanism                                                       |
| ---------- | ------------------------------------------------------------------------ |
| Rust       | `std::any::TypeId`                                                       |
| C++        | `std::type_index` (from `<typeindex>`)                                   |
| TypeScript | Generic type parameters + `Map<symbol, T>` or class constructors as keys |
| Java       | `Class<T>` objects                                                       |
| Python     | `type` objects                                                           |
| Go         | `reflect.Type`                                                           |

**Key insight**: The registry doesn't store "named" singletons—it stores one instance *per type*. This provides compile-time type safety on retrieval.

### 2. Thread-Safe Storage

All implementations must be thread-safe for concurrent read/write:

| Language      | Recommended Approach                                                               |
| ------------- | ---------------------------------------------------------------------------------- |
| Rust          | `Mutex<HashMap<TypeId, Arc<dyn Any>>>`                                             |
| C++           | `std::shared_mutex` + `std::unordered_map<std::type_index, std::shared_ptr<void>>` |
| TypeScript/JS | Single-threaded by default; use `Atomics` or worker-based isolation if needed      |
| Java          | `ConcurrentHashMap<Class<?>, Object>`                                              |
| Python        | `threading.Lock` + `dict`                                                          |
| Go            | `sync.RWMutex` + `map[reflect.Type]interface{}`                                    |

### 3. Reference Counting for Safe Replacement

When a singleton is replaced, existing references must remain valid:

| Language   | Reference Mechanism                      |
| ---------- | ---------------------------------------- |
| Rust       | `Arc<T>` — clone before returning        |
| C++        | `std::shared_ptr<T>` — copy on retrieval |
| TypeScript | Object references (GC handles lifetime)  |
| Java       | Object references (GC handles lifetime)  |
| Python     | Object references (GC handles lifetime)  |
| Go         | Pointers + GC                            |

**Critical behavior**: `get()` returns a **clone/copy of the reference**, not a direct pointer into the storage. This ensures that:

1. The caller owns their reference independently
2. Replacement updates storage without invalidating outstanding references
3. Old instances are garbage-collected when all references are dropped

### 4. Replacement Semantics

```()
register(value) → Always overwrites existing entry for that type
get<T>() → Returns cloned reference or error if not found
contains<T>() → Returns boolean
```

**No removal**: By design, singletons cannot be removed—only replaced. This simplifies reasoning about availability.

### 5. Registry Isolation

Support multiple independent registries:

| Language   | Isolation Mechanism                                            |
| ---------- | -------------------------------------------------------------- |
| Rust       | `define_registry!` macro generates separate static storage     |
| C++        | Template class with different tag types, or separate instances |
| TypeScript | Class instances or module-scoped `Map` objects                 |
| Java       | Separate `Registry` class instances                            |

## API Surface

Each registry should expose:

| Function             | Signature (Rust-like)                           | Purpose                                       |
| -------------------- | ----------------------------------------------- | --------------------------------------------- |
| `register`           | `fn register<T>(value: T)`                      | Store a singleton, wrapping in Arc/shared_ptr |
| `register_arc`       | `fn register_arc<T>(arc: Arc<T>)`               | Store pre-wrapped reference                   |
| `get`                | `fn get<T>() -> Result<Arc<T>, Error>`          | Retrieve singleton reference                  |
| `get_cloned`         | `fn get_cloned<T: Clone>() -> Result<T, Error>` | Retrieve owned clone                          |
| `contains`           | `fn contains<T>() -> Result<bool, Error>`       | Check if type is registered                   |
| `set_trace_callback` | `fn set_trace_callback(cb: Fn(Event))`          | Optional observability hook                   |

## Error Handling

Define an error enum/type covering:

- **TypeNotFound**: Requested type not in registry
- **TypeMismatch**: Downcast failed (should be impossible with correct implementation)
- **LockError**: Failed to acquire mutex (recover gracefully if possible)

## Implementation Checklist

- [ ] Type-keyed storage with language-appropriate key mechanism
- [ ] Thread-safe access with mutex/lock
- [ ] Reference-counted values (Arc, shared_ptr, or GC)
- [ ] Clone reference on `get()` before returning
- [ ] Overwrite-on-register behavior
- [ ] Multiple isolated registries support
- [ ] Tracing callback system (optional but recommended)
- [ ] Comprehensive error types
- [ ] Unit tests covering: basic types, trait objects, replacement, thread safety

## Language-Specific Notes

### TypeScript

```typescript
// Using class constructor as key
class Registry {
  private storage = new Map<Function, unknown>();

  register<T>(ctor: new (...args: any[]) => T, value: T): void {
    this.storage.set(ctor, value);
  }

  get<T>(ctor: new (...args: any[]) => T): T | undefined {
    return this.storage.get(ctor) as T | undefined;
  }
}

// Or using Symbol for interface-based keys
const ServiceKey = Symbol('ServiceContract');
registry.register(ServiceKey, myServiceImpl);
```

### C++

```cpp
#include <typeindex>
#include <unordered_map>
#include <shared_mutex>
#include <memory>
#include <any>

class Registry {
    std::shared_mutex mutex_;
    std::unordered_map<std::type_index, std::shared_ptr<void>> storage_;

public:
    template<typename T>
    void register_value(std::shared_ptr<T> value) {
        std::unique_lock lock(mutex_);
        storage_[std::type_index(typeid(T))] = value;
    }

    template<typename T>
    std::shared_ptr<T> get() {
        std::shared_lock lock(mutex_);
        auto it = storage_.find(std::type_index(typeid(T)));
        if (it == storage_.end()) return nullptr;
        return std::static_pointer_cast<T>(it->second);
    }
};
```

## Testing Strategy

1. **Basic registration**: Register primitives, retrieve correctly
2. **Type isolation**: Same value type in different registries remains separate
3. **Replacement**: Re-register overwrites, old references still valid
4. **Thread safety**: Concurrent register/get from multiple threads
5. **Trait objects**: Register interface implementations, retrieve via interface type
6. **Error cases**: Get before register returns appropriate error

## Performance Considerations

- Keep lock hold time minimal (insert/lookup only)
- Consider read-write locks if reads vastly outnumber writes
- Tracing callbacks should be invoked outside the lock to prevent deadlocks
- For hot paths, consider caching retrieved references locally

## License

Implementations derived from this design guidance may use any license. The original Rust crate is BSD-3-Clause.
