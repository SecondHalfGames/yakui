use std::str::FromStr;

use anyhow::bail;

use crate::AppState;

macro_rules! apps {
    ($macro:ident) => {
        $macro!(bench, simple, align, images);
    };
}

macro_rules! define_app {
    ($($mod:ident),* $(,)?) => {
        $(pub mod $mod;)*

        #[allow(non_camel_case_types)]
        pub enum App {
            $($mod,)*
        }

        impl App {
            pub fn function(&self) -> &'static dyn Fn(&AppState) {
                match self {
                    $(App::$mod => &$mod::app,)*
                }
            }
        }

        impl FromStr for App {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(stringify!($mod) => Ok(Self::$mod),)*
                    unknown => {
                        let app_list = [$(stringify!($mod),)*].join(", ");
                        bail!("unknown app '{unknown}', included apps are: {app_list}");
                    },
                }
            }
        }
    }
}

apps!(define_app);
