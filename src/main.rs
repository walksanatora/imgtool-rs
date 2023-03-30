#![allow(unused_parens)]
use std::fs::File;
use std::io::{Write,Read};
use std::ops::Deref;
use serde::{Serialize, Deserialize};
use mlua::prelude::*;
use clap::{Parser,command};
use std::fs::metadata;
use libflate::gzip::{Encoder,Decoder};
use bstr::BString;

macro_rules! out_bytes {
    ($file:expr) => {
        include_bytes!(concat!(env!("OUT_DIR"),"/",$file)).as_slice()
    };
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
#[command(author = "walksanator", version = "2.0.0", about = "ComputerCraft VFS tool", long_about = None)]
struct Args {
    #[arg(short,long,required=true, help = "directory to make/extract to")]
    directory: String,

    #[arg(short,long,default_value = "archive.vfs", help = "archive file to use")]
    archive: String,

    #[arg(short,long,help="is archive compressed with gzip (via zlib)")]
    compressed: bool,

    #[arg(short, long, help = "are we extracting vfs to directory")]
    extract: bool,

    #[arg(short, long, help = "make the archive self-extracting")]
    self_extracting: bool
}

impl mlua::UserData for Args {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("directory", |_,this| Ok(this.directory.clone()));
        fields.add_field_method_get("archive", |_,this| Ok(this.archive.clone()));
        fields.add_field_method_get("extract", |_,this| Ok(this.extract.clone()));
        fields.add_field_method_get("compressed", |_,this| Ok(this.compressed.clone()));
        fields.add_field_method_get("sea", |_,this| Ok(this.self_extracting.clone()));
        fields.add_field_method_get("ld", |_,_|{
            Ok(include_str!("ld.lua"))
        });
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
struct FS {}

impl mlua::UserData for FS {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("read", |_,_this,path: String| {
            let s = std::fs::read(&path);
            if let Ok(str) = s {
                Ok(BString::from(str))
            } else {
                Err(mlua::Error::RuntimeError(
                    format!("Failed to read file {} reason: {}",path,s.unwrap_err())
                ))
            }
        });
        methods.add_method("write", |_,_this,opts: (String,BString)| {
            let s = File::options()
                                            .write(true)
                                            .create(true)
                                            .open(&opts.0);
            if let Ok(mut file) = s {
                let wr = file.write(&opts.1.deref());
                if let Ok(written) = wr {
                    Ok(written)
                } else {
                    Err(mlua::Error::RuntimeError(
                        format!("Failed to write file {} reason: {}",opts.0,wr.unwrap_err())
                    ))
                }
            } else {
                Err(mlua::Error::RuntimeError(
                    format!("Failed to open file {} reason: {}",opts.0,s.unwrap_err())
                ))
            }
        });
        methods.add_method("ungz", |_lua,_this,bytes: BString| {
            let mut decoder = Decoder::new(bytes.as_slice()).unwrap();
            let mut decoded_data: Vec<u8> = Vec::new();
            decoder.read_to_end(&mut decoded_data).unwrap();
            let output = BString::from(decoded_data);
            Ok(output)
        });
        methods.add_method("gz", |lua,_this,bytes: BString| {
            let enc = Encoder::new(Vec::new());
            if let Ok(mut encoder) = enc {
                let encres = encoder.write(bytes.as_slice());
                if let Err(_err) = encres {
                    return Err(LuaError::RuntimeError("Failed to write to encoder".into()))
                };
                let done = encoder.finish();
                let encoded = done.as_result().expect("Failed to unwrap encoder output");
                Ok(BString::from(encoded.clone()).to_lua(lua))
            } else {
                Err(LuaError::RuntimeError("Failed to create gzip encoder".into()))
            }

        });
        
        methods.add_method("list", |_,_this,path: String| {
            let s = std::fs::read_dir(&path);
            if let Ok(rdir) = s {
                let mut paths = vec![];
                for path in rdir {
                    paths.push(path.unwrap().path().display().to_string());
                };
                Ok(paths)
            } else {
                Err(mlua::Error::RuntimeError(
                    format!("Failed to list {} Reason: {}",path,s.unwrap_err())
                ))
            }
        });
        methods.add_method("isdir", |_,_this,path: String| {
            let md = metadata(&path);
            if let Ok(meta) = md {
                Ok(meta.is_dir())
            } else {
                Err(mlua::Error::RuntimeError(format!("Error {}: {}",path,md.unwrap_err())))
            }
        });
        methods.add_method("makedir", |_,_this,path: String| {
            if let Err(e) = std::fs::create_dir_all(path) {
                return Err(LuaError::RuntimeError(format!("{}",e)))
            };
            Ok(())
        });
    }
}

fn main() {

    let args = Args::parse();
    //create a new lua env
    let lenv = Lua::new();

    //create the FS library
    let utype = lenv.create_ser_userdata(FS{}).expect("Unnable to add userdata");
    lenv.globals().set("fs",utype).unwrap();

    //create the loadstring function
    lenv.globals().set("loadstring",
        lenv.create_function(|lua, (code): (String)|{
            let stat = lua.load(&code).into_function();
            if let Ok(func) = stat {
                Ok(func)
            } else {
                Err(stat.unwrap_err())
            }
        }).expect("Unnable to create loadstring function")
    ).expect("unnable to add loadstring to _G");

    //put ARGS into the global table
    lenv.globals().set(
        "ARGS",
        lenv.create_ser_userdata(
            args.clone()
        ).expect("Unnable to create UserData args!")
    ).expect("Failed to add ARGS to _G");

    //run the code in util.lua to add to _G
    lenv.load(out_bytes!("util.lua")).exec().expect("util.lua failed to execute");    
    //sandbox _G from future modification
    lenv.sandbox(true).expect("Failed to sandbox _G lua functions");

    //run ser or de lua files bytecode
    if let Err(ohno) = lenv.load(if args.extract {
        out_bytes!("de.lua") //decompression
    } else {
        out_bytes!("ser.lua") //compression
    }).exec() {
        match ohno { //error handling
            LuaError::RuntimeError(err) => {println!("{}",err)}, //lua script error
            LuaError::CallbackError{traceback, cause} => {println!("{}\n{}",cause,traceback)},
            _ => println!("{:?}",ohno) //other error
        }
    }
    println!("DONE");
}
