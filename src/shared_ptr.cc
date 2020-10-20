#include <memory>

namespace {

struct T;

extern "C" void shared_ptr__construct(T* ptr,
                                      void (*deleter)(T*),
                                      std::shared_ptr<T>* out) {
    new(out) std::shared_ptr<T>(ptr, deleter);
}

extern "C" void shared_ptr__destruct(std::shared_ptr<T>* sp) {
    sp->~shared_ptr();
}

extern "C" void shared_ptr__copy(const std::shared_ptr<T>* sp,
                                 std::shared_ptr<T>* out) {
    new(out) std::shared_ptr<T>();
    *out = *sp;
}

extern "C" T* shared_ptr__get(const std::shared_ptr<T>* sp) {
    return sp->get();
}

}
