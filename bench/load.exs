small = File.read!("bench/small.bin")
medium = File.read!("bench/medium.bin")
large = File.read!("bench/large.bin")

Benchee.run(
  %{
    "erlang" => fn input -> :erlang.binary_to_term(input) end,
    "rust" => fn input -> EtfLoader.from_binary!(input) end
  },
  inputs: %{
    "Small" => small,
    "Medium" => medium,
    "Bigger" => large
  }
)
