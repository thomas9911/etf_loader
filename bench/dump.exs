small = File.read!("bench/small.bin") |> :erlang.binary_to_term()
medium = File.read!("bench/medium.bin") |> :erlang.binary_to_term()
large = File.read!("bench/large.bin") |> :erlang.binary_to_term()

defmodule TestData do
  use ExUnitProperties

  def my_term() do
    simple_term =
      one_of([
        boolean(),
        integer(),
        byte(),
        binary(),
        float(),
        atom(:alphanumeric),
        atom(:alias),
        iolist()
      ])

    tree(simple_term, fn leaf ->
      one_of([
        list_of(leaf),
        map_of(leaf, leaf),
        maybe_improper_list_of(leaf, leaf),
        keyword_of(leaf),
        mapset_of(leaf),
        one_to_four_element_tuple(leaf)
      ])
    end)
  end

  defp one_to_four_element_tuple(leaf) do
    bind(integer(0..12), fn
      int when int >= 9 -> leaf
      int when int >= 6 -> {leaf, leaf, leaf}
      int when int >= 3 -> {leaf, leaf}
      int when int >= 1 -> {leaf}
      _ -> {}
    end)
  end
end

Benchee.run(
  %{
    "erlang" => fn input -> :erlang.term_to_binary(input) end,
    "rust" => fn input -> EtfLoader.to_binary!(input) end
  },
  inputs: %{
    "Small" => small,
    "Medium" => medium,
    "Bigger" => large
  }
)
