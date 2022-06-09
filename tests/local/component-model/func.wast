(component
  (import "" (func (param "foo" string)))
  (import "a" (func (param "foo" string) (param s32) (param "bar" u32)))
)

(assert_invalid
  (component
      (core module $m
          (memory (export "memory") 1)
          (func (export "foo") (result i32) unreachable)
      )
      (core instance $i (instantiate $m))

      (func (export "tuple") (result (tuple s8 u8))
          (canon lift (core func $i "foo"))
      )
  )
  "canonical option `memory` is required"
)

(component
  (import "" (func $log (param string)))
  (core module $libc
    (memory (export "memory") 1)
  )
  (core instance $libc (instantiate $libc))
  (core func (canon lower (func $log) (memory (core memory $libc "memory"))))
)

(component
  (core module $m
    (memory (export "memory") 1)
    (func (export "ret-list") (result i32) unreachable)
  )
  (core instance $i (instantiate $m))

  (func (export "ret-list") (result (list u8))
    (canon lift (core func $i "ret-list") (memory (core memory $i "memory")))
  )
)

(assert_invalid
  (component
    (import "" (func $log (result string)))
    (core module $libc
      (memory (export "memory") 1)
    )
    (core instance $libc (instantiate $libc))
    (core func (canon lower (func $log) (memory (core memory $libc "memory"))))
  )
  "canonical option `realloc` is required"
)

(assert_invalid
  (component
    (core module $m
      (memory (export "memory") 1)
      (func (export "param-list") (param i32 i32) unreachable)
    )
    (core instance $i (instantiate $m))

    (func (export "param-list") (param (list u8))
      (canon lift (core func $i "param-list") (memory (core memory $i "memory")))
    )
  )
  "canonical option `realloc` is required"
)
