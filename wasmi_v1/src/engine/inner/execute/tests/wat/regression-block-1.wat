;; Regression test testing returning values from a basic block.
(module
  (func (export "func") (result i32)
    (block (result i32) (i32.const 7))
  )
)