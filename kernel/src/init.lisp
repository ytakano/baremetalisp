(export spawn (id) (IO (-> (Int) (Option Int)))
    (call-rust 0 id 0))