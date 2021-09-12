defmodule EtfLoaderTest do
  use ExUnit.Case
  use ExUnitProperties

  doctest EtfLoader

  def my_term() do
    # copied from the StreamData.term/0 
    # except removes reference and adds extra data structures
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

  def assert_term_same(input) do
    output =
      input
      |> EtfLoader.to_binary!()
      |> :erlang.binary_to_term()

    assert output == input
  end

  test "float" do
    data = EtfLoader.to_binary!(1.123)
    assert <<131, 70, 63, 241, 247, 206, 217, 22, 135, 43>> == data

    assert :erlang.binary_to_term(data) == 1.123
  end

  test "map" do
    assert_term_same %{"test" => 1}
  end

  test "atom map" do
    assert_term_same %{test: 1}
  end

  test "tuple" do
    assert_term_same {1, 2, 3}
  end

  test "string" do
    assert_term_same "testing some text"
  end

  test "binary" do
    assert_term_same <<1, 2, 3>>
  end

  test "charlist" do
    assert_term_same 'testing some text'
  end

  test "list" do
    assert_term_same [:value, 1, true, "testing"]
  end

  test "keylist" do
    assert_term_same test: :value, more: "tests"
  end

  test "large_number" do
    assert_term_same 100_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000
  end

  test "nil" do
    assert_term_same nil
  end

  test "empty tuple" do
    assert_term_same {}
  end

  test "[<<203>>]" do
    assert_term_same [<<203>>]
  end

  test "improper list" do
    assert_term_same [1 | 1]
  end

  test "bitstring errors" do
    assert {:error,
            %EtfLoader.Error{
              message: "cannot format term of type: 'Unknown'",
              type: "Unknown"
            }} = EtfLoader.to_binary(<<0::size(1)>>)
  end

  test "reference errors" do
    assert {:error, %EtfLoader.Error{message: "cannot format term of type: 'Ref'", type: "Ref"}} =
             EtfLoader.to_binary(make_ref())
  end

  test "function errors" do
    func = fn x -> x * 2 end

    assert {:error, %EtfLoader.Error{message: "cannot format term of type: 'Fun'", type: "Fun"}} =
             EtfLoader.to_binary(func)
  end

  property "works for all terms" do
    check all data <- my_term() do
      assert_term_same data
    end
  end
end
