#![allow(unused_parens)]
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;
use serde::{Serialize, Deserialize};
use mlua::prelude::*;
use clap::{Parser,command};
use std::fs::metadata;

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
#[command(author = "walksanator", version = "1.0.0", about = "ComputerCraft VFS tool", long_about = None)]
struct Args {
    #[arg(short,long,default_value = "src", help = "directory to make/extract to")]
    directory: String,

    #[arg(short,long,default_value = "archive.vfs", help = "archive file to use")]
    archive: String,

    #[arg(short, long, help = "are we extracting vfs to directory")]
    extract: bool
}

impl mlua::UserData for Args {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("directory", |_,this| Ok(this.directory.clone()));
        fields.add_field_method_get("archive", |_,this| Ok(this.archive.clone()));
        fields.add_field_method_get("extract", |_,this| Ok(this.extract.clone()));
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
struct FS {}

impl mlua::UserData for FS {
    //fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
    //    fields.add_field_method_get("num1", |_, this| Ok(this.num1));
    //}
    //TODO: mkdir, isdir
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("read", |_,_this,path: String| {
            let s = read_to_string(&path);
            if let Ok(str) = s {
                Ok(str)
            } else {
                Err(mlua::Error::RuntimeError(
                    format!("Failed to read file {} reason: {}",path,s.unwrap_err())
                ))
            }
        });
        methods.add_method("write", |_,_this,opts: (String,Box<str>)| {
            let s = File::options()
                                            .write(true)
                                            .create(true)
                                            .open(&opts.0);
            if let Ok(mut file) = s {
                let wr = file.write(&opts.1.into_boxed_bytes());
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
            std::fs::create_dir_all(path);
            Ok(())
        });
    }
}

fn main() {

    let args = Args::parse();
    //create a new lua env
    let lenv = Lua::new();
    lenv.globals().set("nums",(1..100).collect::<Vec<u8>>()).expect("Failed to add nums global");

    let test_function = lenv.create_function(|_lua,(s): (String)|{
        println!("RUST: {}",s);
        Ok(())
    }).expect("Failed to create lua function");

    let utype = lenv.create_ser_userdata(FS{}).expect("Unnable to add userdata");
    lenv.globals().set("fs",utype).unwrap();
    lenv.globals().set("print",test_function).expect("Unnable to add test function as tf");

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

    lenv.globals().set(
        "ARGS",
        lenv.create_ser_userdata(
            args.clone()
        ).expect("Unnable to create UserData args!")
    ).expect("Failed to add ARGS to _G");

    lenv.load(include_str!("lua/util.lua")).exec().expect("util.lua failed to execute");    
    lenv.sandbox(true).expect("Failed to sandbox _G lua functions");
    println!("ENV setup");
    if !args.extract {
        if let Err(ohno) = lenv.load(include_str!("lua/ser.lua")).exec() {
            println!("{:?}",ohno)
        }
    } else {
        if let Err(ohno) = lenv.load(include_str!("lua/de.lua")).exec() {
            println!("{:?}",ohno)
        }
    }
    println!("DONE");
}
