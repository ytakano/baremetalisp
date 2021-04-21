(export spawn (app) (IO (-> (Int) (Option Int)))
    (call-rust 1 app 0))

(export sched_yield () (IO (-> () []))
    (let ((_ (call-rust 3 0 0)))
        []))

(export getpid () (IO (-> () Int))
    (match (call-rust 4 0 0)
        ((Some id) id)
        (_ 0))) ; unreachable
