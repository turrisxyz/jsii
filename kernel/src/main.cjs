'use strict';

const { inspect } = require('util');

function main(logger) {
  logger(`[jsii/kernel] Start time ${new Date().toISOString()}`);

  class Kernel {
    #modules;

    constructor() {
      this.#modules = new Map();
    }

    load(name, path) {
      logger(`[jsii/kernel] load(${name}, ${path})`);
      if (this.#modules.has(name)) {
        return;
      }
      this.#modules.set(name, require(path));
    }

    create(fqn, args) {
      logger(`[jsii/kernel] create(${[fqn, ...args.map((arg) => inspect(arg, false, 1))].join(', ')})`);
      if (fqn === "Object") {
        return Object.new(null);
      }
      const [moduleName, ...path] = fqn.split(".");
      const module = this.#modules.get(moduleName);
      if (module == null) {
        throw new Error(`Module ${moduleName} not found. Was it loaded?`);
      }
      const ctor = path.reduce((exports, name) => exports?.[name], module);
      if (ctor == null) {
        throw new Error(`Could not find constructor for ${path.join('.')} in ${moduleName}`);
      }

      const result = new ctor(...args);
      logger(`[jsii/kernel] ==> ${inspect(result, false, 1)}`);
      return result;
    }

    call(recv, methodName, args) {
      logger(`[jsii/kernel] call(${[recv, methodName, ...args].map((arg) => inspect(arg, false, 1)).join(', ')})`);
      const method = recv[methodName];
      if (method == null) {
        throw new Error(`No method named ${methodName} found on receiver with type ${recv.consturctor.name}`);
      }
      if (methodName === "metricErrors") {
        debugger;
      }
      const result = method.call(recv, ...args);
      logger(`[jsii/kernel] ==> ${inspect(result, false, 1)}`);
      return result;
    }

    call_static(fqn, methodName, args) {
      logger(`[jsii/kernel] call(${[fqn, methodName, ...args].map((arg) => inspect(arg, false, 1)).join(', ')})`);

      const [moduleName, ...path] = fqn.split(".");
      const module = this.#modules.get(moduleName);
      if (module == null) {
        throw new Error(`Module ${moduleName} not found. Was it loaded?`);
      }
      const ctor = path.reduce((exports, name) => exports?.[name], module);
      if (ctor == null) {
        throw new Error(`Could not find constructor for ${path.join('.')} in ${moduleName}`);
      }
      const method = ctor[methodName];
      if (method == null) {
        throw new Error(`No method named ${methodName} found on ${fqn}`);
      }
      const result = method.call(ctor, ...args);
      logger(`[jsii/kernel] ==> ${inspect(result, false, 1)}`);
      return result;
    }

    get(recv, propertyName) {
      logger(`[jsii/kernel] get(${[recv, propertyName].map((arg) => inspect(arg, false, 1)).join(', ')})`);

      const result = recv[propertyName];
      logger(`[jsii/kernel] ==> ${inspect(result, false, 1)}`);
      return result;
    }

    get_static(fqn, propertyName) {
      logger(`[jsii/kernel] get_static(${[fqn, propertyName].map((arg) => inspect(arg, false, 1)).join(', ')})`);

      const [moduleName, ...path] = fqn.split(".");
      const module = this.#modules.get(moduleName);
      if (module == null) {
        throw new Error(`Module ${moduleName} not found. Was it loaded?`);
      }
      const ctor = path.reduce((exports, name) => exports?.[name], module);
      if (ctor == null) {
        throw new Error(`Could not find constructor for ${path.join('.')} in ${moduleName}`);
      }
      const result = ctor[propertyName];
      logger(`[jsii/kernel] ==> ${inspect(result, false, 1)}`);
      return result;
    }

    set(recv, propertyName, value) {
      logger(`[jsii/kernel] set(${[recv, propertyName, value].map((arg) => inspect(arg, false, 1)).join(', ')})`);

      recv[propertyName] = value;
    }
  }

  global.jsii = new Kernel();
}

// For debugging   => main(console.error);
// For productiion => main(() => undefined);
main(console.error);
