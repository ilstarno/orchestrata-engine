{
  "variables": {
    "orchestrata_lib_dir%": "<!(node -p \"process.env.ORCHESTRATA_LIB_DIR || require('node:path').resolve('../../target/release')\")",
    "orchestrata_include_dir%": "<!(node -p \"process.env.ORCHESTRATA_INCLUDE_DIR || require('node:path').resolve('../../crates/ffi/include')\")"
  },
  "targets": [{
    "target_name": "orchestrata",
    "sources": ["native.c"],
    "conditions": [
      ["OS=='linux'", {
        "cflags": ["<!(node linux-build-flag.js include)"],
        "libraries": ["<!(node linux-build-flag.js library)", "-lorchestrata_ffi"]
      }, {
        "include_dirs": ["<(orchestrata_include_dir)"]
      }],
      ["OS=='win'", {
        "libraries": ["<(orchestrata_lib_dir)/orchestrata_ffi.dll.lib"]
      }],
      ["OS!='linux' and OS!='win'", {
        "libraries": ["-L<(orchestrata_lib_dir)", "-lorchestrata_ffi"]
      }]
    ]
  }]
}
