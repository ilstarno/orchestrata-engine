{
  "variables": {
    "orchestrata_lib_dir%": "<!(node -p \"process.env.ORCHESTRATA_LIB_DIR || require('node:path').resolve('../../target/release')\")",
    "orchestrata_include_dir%": "<!(node -p \"process.env.ORCHESTRATA_INCLUDE_DIR || require('node:path').resolve('../../crates/ffi/include')\")"
  },
  "targets": [{
    "target_name": "orchestrata",
    "sources": ["native.c"],
    "include_dirs": ["<(orchestrata_include_dir)"],
    "conditions": [
      ["OS=='win'", {
        "libraries": ["<(orchestrata_lib_dir)/orchestrata_ffi.dll.lib"]
      }, {
        "libraries": ["-L<(orchestrata_lib_dir)", "-lorchestrata_ffi"]
      }]
    ]
  }]
}
