# Used by "mix format"
[
  inputs: ["{mix,.formatter}.exs", "{config,lib,test}/**/*.{ex,exs}"],
  locals_without_parens: [assert_to_term_same: 1, assert_from_term_same: 1],
  import_deps: [:stream_data]
]
