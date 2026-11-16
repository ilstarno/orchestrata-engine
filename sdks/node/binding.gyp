{
  "variables": {
    "orchestrata_lib_dir%": "<!(node -p \"process.env.ORCHESTRATA_LIB_DIR || require('node:path').resolve('../../backend/target/release')\")"
  },
  "targets": [{
    "target_name": "orchestrata",
    "sources": ["native.c"],
    "include_dirs": ["../../backend/crates/ffi/include"],
    "conditions": [
      ["OS=='win'", {
        "libraries": ["<(orchestrata_lib_dir)/orchestrata_ffi.dll.lib"]
      }, {
        "libraries": ["-L<(orchestrata_lib_dir)", "-lorchestrata_ffi"]
      }]
    ]
  }]
}
