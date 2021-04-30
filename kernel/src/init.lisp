(export spawn (app) (IO (-> (Int) (Option Int)))
    (call-rust 1 app 0))

(export sched_yield () (IO (-> () []))
    (let ((_ (call-rust 3 0 0)))
        []))

(export getpid () (IO (-> () Int))
    (match (call-rust 4 0 0)
        ((Some id) id)
        (_ 0))) ; unreachable

(export send (dst val) (IO (-> (Int Int) Bool))
    (match (call-rust 5 dst val)
        ((Some _) true)
        (_ false)))

(export recv () (IO (-> () Int))
    (match (call-rust 6 0 0)
        ((Some val) val)
        (_ 0))) ; unreachable
