macro_rules! fn_bench {
    ($typ:ident) => {
        fn_bench_inner!($typ, $typ, bench);
    };
}

macro_rules! fn_bench_multithread {
    ($typ:ident, $fn_name:ident) => {
        fn_bench_inner!($typ, $fn_name, bench_multithread);
    };
}

macro_rules! fn_bench_inner {
    ($typ:ident, $name:ident, $fn:ident) => {
        pub fn $name (&self, config: $typ::Config) -> Result<$typ::Report, $typ::Error> {
            $typ::$fn(&self.features, config)
        }
    };
}

pub(crate) use fn_bench_inner;

pub(crate) use fn_bench;
pub(crate) use fn_bench_multithread;
