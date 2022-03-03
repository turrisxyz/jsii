use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::{fs, path::Path, sync::Arc};

use anyhow::{anyhow, Result};
use deno::deno_core;
use deno::deno_runtime;
pub use deno::v8;
use tempdir::TempDir;

pub struct Kernel {
  api: KernelApi,
  worker: deno_runtime::worker::MainWorker,
  workdir: TempDir,
}

impl Kernel {
  pub async fn new() -> Result<Self> {
    let workdir = TempDir::new("jsii.rs")?;
    let main = {
      let main = workdir.path().join("index.cjs");
      fs::write(&main, include_bytes!("main.cjs"))?;
      main.to_string_lossy().to_string()
    };

    let run_flags = deno::RunFlags { script: main.clone() };
    let flags = deno::Flags {
      argv: vec![],
      subcommand: deno::DenoSubcommand::Run(run_flags),
      allow_all: false,
      allow_env: Some(vec![]),
      allow_hrtime: true,
      allow_net: None,
      allow_ffi: None,
      allow_read: Some(vec![]),
      allow_run: None,
      allow_write: Some(vec![]),
      ca_stores: None,
      ca_file: None,
      cache_blocklist: vec![],
      cache_path: None,
      cached_only: false,
      check: deno::CheckFlag::None,
      config_path: None,
      coverage_dir: None,
      enable_testing_features: true,
      ignore: vec![],
      import_map_path: None,
      inspect_brk: None,
      inspect: Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 9229))),
      location: None,
      lock_write: false,
      lock: None,
      log_level: None,
      no_remote: false,
      compat: true,
      no_prompt: true,
      reload: false,
      repl: false,
      seed: None,
      unstable: true,
      unsafely_ignore_certificate_errors: None,
      v8_flags: vec![],
      version: false,
      watch: None,
      no_clear_screen: true,
    };
    let main = deno::deno_core::resolve_url_or_path(&main)?;
    let state = deno::ProcState::build(Arc::new(flags)).await?;

    let permissions = deno::Permissions::from_options(&state.flags.permissions_options());

    let mut worker = deno::create_main_worker(&state, main.clone(), permissions, vec![]);

    worker.execute_side_module(&deno::compat::GLOBAL_URL).await?;
    worker.execute_side_module(&deno::compat::MODULE_URL).await?;

    deno::compat::load_cjs_module(&mut worker.js_runtime, &main.to_file_path().unwrap().display().to_string(), true)?;

    worker.dispatch_load_event(&deno_core::located_script_name!())?;
    worker.run_event_loop(false).await?;
    worker.dispatch_unload_event(&deno_core::located_script_name!())?;

    let api = KernelApi::from(&mut worker.js_runtime);

    Ok(Self { api, worker, workdir })
  }

  pub fn handle_scope(&mut self) -> v8::HandleScope {
    self.worker.js_runtime.handle_scope()
  }

  pub async fn load(&mut self, name: &str, version: &str, tarball_path: &Path) -> Result<()> {
    {
      let main_path = self.extract_tarball(tarball_path, name, version)?;

      let scope = &mut self.worker.js_runtime.handle_scope();
      let load = v8::Local::new(scope, &self.api.load);

      let jsii = v8::Local::new(scope, &self.api.jsii).into();
      let name = v8::String::new(scope, name).unwrap().into();
      let main_path = v8::String::new(scope, &main_path).unwrap().into();

      load.call(scope, jsii, &[name, main_path]);
    }

    self.worker.run_event_loop(false).await?;

    Ok(())
  }

  pub fn create(&mut self, fqn: &str, args: Vec<v8::Global<v8::Value>>) -> Result<v8::Global<v8::Object>> {
    let scope = &mut self.worker.js_runtime.handle_scope();
    let create = v8::Local::new(scope, &self.api.create);

    let result = {
      let jsii = v8::Local::new(scope, &self.api.jsii).into();
      let fqn = v8::String::new(scope, fqn).unwrap().into();
      let args = {
        let arr = v8::Array::new(scope, args.len().try_into().unwrap());
        for (i, arg) in args.iter().enumerate() {
          let arg = v8::Local::new(scope, arg);
          arr.set_index(scope, i.try_into().unwrap(), arg).unwrap();
        }
        arr.into()
      };
      create.call(scope, jsii, &[fqn, args])
    };

    match result {
      Some(value) => {
        let value = value.to_object(scope).unwrap();
        Ok(v8::Global::new(scope, value))
      }
      None => Err(anyhow!("Failed to create {} instance", fqn)),
    }
  }

  pub fn call(&mut self, recv: v8::Global<v8::Value>, method: &str, args: Vec<v8::Global<v8::Value>>) -> Result<v8::Global<v8::Value>> {
    let scope = &mut self.worker.js_runtime.handle_scope();
    let call = v8::Local::new(scope, &self.api.call);

    let result = {
      let jsii = v8::Local::new(scope, &self.api.jsii).into();
      let recv = v8::Local::new(scope, recv);
      let method = v8::String::new(scope, method).unwrap().into();
      let args = {
        let arr = v8::Array::new(scope, args.len().try_into().unwrap());
        for (i, arg) in args.iter().enumerate() {
          let arg = v8::Local::new(scope, arg);
          arr.set_index(scope, i.try_into().unwrap(), arg).unwrap();
        }
        arr.into()
      };
      call.call(scope, jsii, &[recv, method, args])
    };

    match result {
      Some(value) => Ok(v8::Global::new(scope, value)),
      None => Err(anyhow!("Failed to call {}", method)),
    }
  }

  pub fn call_static(&mut self, fqn: &str, method: &str, args: Vec<v8::Global<v8::Value>>) -> Result<v8::Global<v8::Value>> {
    let scope = &mut self.worker.js_runtime.handle_scope();
    let call_static = v8::Local::new(scope, &self.api.call_static);

    let result = {
      let jsii = v8::Local::new(scope, &self.api.jsii).into();
      let fqn = v8::String::new(scope, fqn).unwrap().into();
      let method = v8::String::new(scope, method).unwrap().into();
      let args = {
        let arr = v8::Array::new(scope, args.len().try_into().unwrap());
        for (i, arg) in args.iter().enumerate() {
          let arg = v8::Local::new(scope, arg);
          arr.set_index(scope, i.try_into().unwrap(), arg).unwrap();
        }
        arr.into()
      };
      call_static.call(scope, jsii, &[fqn, method, args])
    };

    match result {
      Some(value) => Ok(v8::Global::new(scope, value)),
      None => Err(anyhow!("Failed to call static {}", method)),
    }
  }

  pub fn get(&mut self, recv: v8::Global<v8::Value>, property: &str) -> Result<v8::Global<v8::Value>> {
    let scope = &mut self.worker.js_runtime.handle_scope();
    let get = v8::Local::new(scope, &self.api.get);

    let result = {
      let jsii = v8::Local::new(scope, &self.api.jsii).into();
      let recv = v8::Local::new(scope, recv);
      let property = v8::String::new(scope, property).unwrap().into();
      get.call(scope, jsii, &[recv, property])
    };

    match result {
      Some(value) => Ok(v8::Global::new(scope, value)),
      None => Err(anyhow!("Failed to get static {}", property)),
    }
  }

  pub fn get_static(&mut self, fqn: &str, property: &str) -> Result<v8::Global<v8::Value>> {
    let scope = &mut self.worker.js_runtime.handle_scope();
    let get_static = v8::Local::new(scope, &self.api.get_static);

    let result = {
      let jsii = v8::Local::new(scope, &self.api.jsii).into();
      let fqn = v8::String::new(scope, fqn).unwrap().into();
      let property = v8::String::new(scope, property).unwrap().into();
      get_static.call(scope, jsii, &[fqn, property])
    };

    match result {
      Some(value) => Ok(v8::Global::new(scope, value)),
      None => Err(anyhow!("Failed to get static {}", property)),
    }
  }

  pub fn set(&mut self, recv: v8::Global<v8::Value>, property: &str, value: v8::Global<v8::Value>) -> Result<()> {
    let scope = &mut self.worker.js_runtime.handle_scope();
    let set = v8::Local::new(scope, &self.api.set);

    let result = {
      let jsii = v8::Local::new(scope, &self.api.jsii).into();
      let recv = v8::Local::new(scope, recv);
      let property = v8::String::new(scope, property).unwrap().into();
      let value = v8::Local::new(scope, value);
      set.call(scope, jsii, &[recv, property, value])
    };

    match result {
      Some(_) => Ok(()),
      None => Err(anyhow!("Failed to set static {}", property)),
    }
  }

  fn extract_tarball(&self, tarball_path: &Path, name: &str, version: &str) -> Result<String> {
    let file = fs::File::open(tarball_path)?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(gz);

    let tempdir = TempDir::new(&name.replace("\\", "/").replace("/", "__"))?;
    tar.unpack(tempdir.path())?;

    let package_dir = tempdir.path().join("package");
    let package_json = package_dir.join("package.json");

    #[derive(serde::Deserialize)]
    struct PackageJson {
      name: String,
      version: String,
      main: Option<String>,
    }

    let metadata: PackageJson = serde_json::from_reader(fs::File::open(package_json)?)?;

    if name != metadata.name || version != metadata.version {
      Err(anyhow!(
        "Name and/or version mismatch. Expected {}@{}, found {}@{} in {:?}",
        name,
        version,
        metadata.name,
        metadata.version,
        tarball_path
      ))
    } else {
      let install_dir = self.workdir.path().join("node_modules").join(name);
      fs::create_dir_all(install_dir.parent().unwrap())?;
      fs::rename(package_dir, &install_dir)?;

      Ok(
        install_dir
          .join(metadata.main.unwrap_or("index.js".to_string()))
          .to_str()
          .unwrap()
          .to_string(),
      )
    }
  }
}

struct KernelApi {
  jsii: v8::Global<v8::Object>,

  load: v8::Global<v8::Function>,
  create: v8::Global<v8::Function>,
  call: v8::Global<v8::Function>,
  call_static: v8::Global<v8::Function>,
  get: v8::Global<v8::Function>,
  get_static: v8::Global<v8::Function>,
  set: v8::Global<v8::Function>,
}

impl KernelApi {
  fn from(runtime: &mut deno_core::JsRuntime) -> Self {
    let context = runtime.global_context();
    let scope = &mut runtime.handle_scope();
    let global = {
      let context = context.open(scope);
      context.global(scope)
    };

    let jsii = {
      let jsii = v8::String::new_external_onebyte_static(scope, b"jsii").unwrap().into();
      let jsii = global.get(scope, jsii).unwrap();
      jsii.to_object(scope).unwrap()
    };
    let load = {
      let load = v8::String::new_external_onebyte_static(scope, b"load").unwrap().into();
      let load = jsii.get(scope, load).unwrap();
      let load: v8::Local<v8::Function> = load.try_into().unwrap();
      v8::Global::new(scope, load)
    };
    let create = {
      let create = v8::String::new_external_onebyte_static(scope, b"create").unwrap().into();
      let create = jsii.get(scope, create).unwrap();
      let create: v8::Local<v8::Function> = create.try_into().unwrap();
      v8::Global::new(scope, create)
    };
    let call = {
      let call = v8::String::new_external_onebyte_static(scope, b"call").unwrap().into();
      let call = jsii.get(scope, call).unwrap();
      let call: v8::Local<v8::Function> = call.try_into().unwrap();
      v8::Global::new(scope, call)
    };
    let call_static = {
      let call_static = v8::String::new_external_onebyte_static(scope, b"call_static").unwrap().into();
      let call_static = jsii.get(scope, call_static).unwrap();
      let call_static: v8::Local<v8::Function> = call_static.try_into().unwrap();
      v8::Global::new(scope, call_static)
    };
    let get = {
      let get = v8::String::new_external_onebyte_static(scope, b"get").unwrap().into();
      let get = jsii.get(scope, get).unwrap();
      let get: v8::Local<v8::Function> = get.try_into().unwrap();
      v8::Global::new(scope, get)
    };
    let get_static = {
      let get_static = v8::String::new_external_onebyte_static(scope, b"get_static").unwrap().into();
      let get_static = jsii.get(scope, get_static).unwrap();
      let get_static: v8::Local<v8::Function> = get_static.try_into().unwrap();
      v8::Global::new(scope, get_static)
    };
    let set = {
      let set = v8::String::new_external_onebyte_static(scope, b"set").unwrap().into();
      let set = jsii.get(scope, set).unwrap();
      let set: v8::Local<v8::Function> = set.try_into().unwrap();
      v8::Global::new(scope, set)
    };

    let jsii = v8::Global::new(scope, jsii);

    Self {
      jsii,
      load,
      create,
      call,
      call_static,
      get,
      get_static,
      set,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::path;

  use anyhow::Result;

  #[tokio::test]
  async fn usage() -> Result<()> {
    let mut kernel = Kernel::new().await?;

    kernel
      .load(
        "constructs",
        "10.0.74",
        &path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/constructs@10.0.74.jsii.tgz"),
      )
      .await?;

    Ok(())
  }
}
