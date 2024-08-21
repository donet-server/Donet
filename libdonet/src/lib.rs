//! DONET SOFTWARE
//!
//! Copyright (c) 2024, Donet Authors.
//!
//! This program is free software; you can redistribute it and/or modify
//! it under the terms of the GNU Affero General Public License version 3.
//! You should have received a copy of this license along
//! with this source code in a file named "LICENSE."
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU Affero General Public License
//! along with this program; if not, write to the Free Software Foundation,
//! Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//!
//! <img src="https://gitlab.com/donet-server/donet/-/raw/master/logo/donet_banner.png?ref_type=heads" height=10%>
//!
//! You can return to the main website for Donet at [`www.donet-server.org`].
//!
//! # libdonet
//! Provides the necessary utilities and definitions for using the Donet networking protocol.
//!
//! These utilities include a lexer, parser, and high-level representation of the parsed DC
//! file, as well as creating datagrams, iterating through datagrams, and the definition of
//! every message type in the Donet protocol.
//!
//! ### Getting Started
//! The recommended way to get started is to enable all features.
//! Do this by enabling the `full` feature flag:
//! ```toml
//! libdonet = { version = "0.1.0", features = ["full"] }
//! ```
//!
//! ### Feature Flags
//! The crate provides a set of feature flags to reduce the amount of compiled code.
//! It is possible to just enable certain features over others.
//! Below is a list of the available feature flags.
//!
//! - **`full`**: Enables all feature flags available for libdonet.
//! - **`datagram`**: Includes Datagram / Datagram Iterator source for writing network packets.
//! - **`dcfile`**: Includes the DC file lexer, parser, and DC element structures.
//!
//! [`www.donet-server.org`]: https://www.donet-server.org/

#![doc(
    html_logo_url = "https://gitlab.com/donet-server/donet/-/raw/master/logo/donet_logo_v3.png?ref_type=heads"
)]
#![allow(clippy::module_inception)]
//#![warn(missing_docs)]
#![deny(unused_extern_crates)]

pub mod globals;

#[macro_use]
extern crate cfg_if;

#[cfg(feature = "datagram")]
pub mod datagram;

cfg_if! {
    if #[cfg(feature = "dcfile")] {
        mod parser;
        pub mod dcarray;
        pub mod dcatomic;
        pub mod dcfield;
        pub mod dcfile;
        pub mod dckeyword;
        pub mod dclass;
        pub mod dcmolecular;
        pub mod dcnumeric;
        pub mod dcparameter;
        pub mod dcstruct;
        pub mod dctype;
        mod hashgen;
    }
}

/// Returns false if a [`log`] logger is not initialized.
///
/// [`log`]: https://docs.rs/log/latest/log/
///
fn logger_initialized() -> bool {
    use log::Level::*;

    let levels: &[log::Level] = &[Error, Warn, Info, Debug, Trace];

    for level in levels {
        if log::log_enabled!(*level) {
            return true;
        }
    }
    false
}

/// Creates a [`pretty_env_logger`] logger if no [`log`]
/// logger is found to be initialized in this process.
///
/// [`pretty_env_logger`]: https://docs.rs/pretty_env_logger/latest/pretty_env_logger/
/// [`log`]: https://docs.rs/log/latest/log/
///
fn init_logger() {
    if logger_initialized() {
        return;
    }
    pretty_env_logger::init();
}

/// Easy to use interface for the DC file parser. Handles reading
/// the DC files, instantiating the lexer and parser, and either
/// returns the DCFile object or a Parse/File error.
///
/// ## Example Usage
/// The following is an example of parsing a simple DC file string,
/// printing its DC hash in hexadecimal notation, and accessing
/// the elements of a defined Distributed Class:
/// ```rust
/// use libdonet::dclass::DClass;
/// use libdonet::globals::DCReadResult;
/// use libdonet::read_dc_files;
///
/// use std::cell::RefCell;
/// use std::rc::Rc;
///
/// let dc_file = "from game.ai import AnonymousContact/UD
///                from game.ai import LoginManager/AI
///                from game.world import DistributedWorld/AI
///                from game.avatar import DistributedAvatar/AI/OV
///
///                dclass AnonymousContact {
///                  login(string username, string password) clsend airecv;
///                };
///
///                dclass LoginManager {
///                  login(channel client, string username, string password) airecv;
///                };
///
///                dclass DistributedWorld {
///                  create_avatar(channel client) airecv;
///                };
///
///                dclass DistributedAvatar {
///                   set_xyzh(int16 x, int16 y, int16 z, int16 h) broadcast required;
///                   indicate_intent(int16 / 10, int16 / 10) ownsend airecv;
///                };";
///
/// let dc_read: DCReadResult = read_dc_files(vec![dc_file.into()]);
///
/// if let Ok(dc_file) = dc_read {
///     // Print the DC File's 32-bit hash in hexadecimal format.
///     println!("{}", dc_file.borrow_mut().get_pretty_hash());
///     
///     // Retrieve the `DistributedAvatar` dclass by ID.
///     let mut avatar_class = dc_file.borrow_mut().get_dclass_by_id(3);
///
///     // Print the identifier of the dclass.
///     println!("{}", Rc::get_mut(&mut avatar_class).expect("Borrow failed!").get_name());
/// }
/// ```
///
/// The output of the program would be the following:
/// ```txt
/// 0x01a5fb0c
/// DistributedAvatar
/// ```
/// <br><img src="https://c.tenor.com/myQHgyWQQ9sAAAAd/tenor.gif">
///
#[cfg(feature = "dcfile")]
pub fn read_dc_files(file_paths: Vec<String>) -> globals::DCReadResult {
    use crate::parser::lexer::Lexer;
    use crate::parser::parser::parse;
    use log::{error, info};
    use std::cell::RefCell;
    use std::fs::File;
    use std::io::Read;
    use std::rc::Rc;

    init_logger();
    info!("DC read of {:?}", file_paths);

    let mut file_results: Vec<Result<File, std::io::Error>> = vec![];
    // All DC files are passed to the lexer as one string.
    let mut lexer_input: String = String::new();

    assert!(!file_paths.is_empty(), "No DC files given!");

    for file_path in &file_paths {
        file_results.push(File::open(file_path));
    }

    for io_result in file_results {
        if let Ok(mut dcf) = io_result {
            let res: std::io::Result<usize> = dcf.read_to_string(&mut lexer_input);
            if let Err(res_err) = res {
                // DC file content may not be in proper UTF-8 encoding.
                return Err(globals::DCReadError::FileError(res_err));
            }
        } else {
            // Failed to open one of the DC files. (most likely permission error)
            return Err(globals::DCReadError::FileError(io_result.unwrap_err()));
        }
    }

    let lexer: Lexer<'_> = Lexer::new(&lexer_input);
    let res: Result<Rc<RefCell<dcfile::DCFile>>, globals::ParseError> = parse(lexer);

    if let Ok(res_ok) = res {
        Ok(res_ok)
    } else {
        Err(globals::DCReadError::ParseError(res.unwrap_err()))
    }
}
