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

  def assert_from_term_same(input) do
    output =
      input
      |> EtfLoader.to_binary!()
      |> :erlang.binary_to_term()

    assert output == input
  end

  def assert_to_term_same(input) do
    output =
      input
      |> :erlang.term_to_binary()
      |> EtfLoader.from_binary!()

    assert output == input
  end

  describe "to_binary" do
    test "float" do
      data = EtfLoader.to_binary!(1.123)
      assert <<131, 70, 63, 241, 247, 206, 217, 22, 135, 43>> == data

      assert :erlang.binary_to_term(data) == 1.123
    end

    test "map" do
      assert_from_term_same %{"test" => 1}
    end

    test "atom map" do
      assert_from_term_same %{test: 1}
    end

    test "tuple" do
      assert_from_term_same {1, 2, 3}
    end

    test "string" do
      assert_from_term_same "testing some text"
    end

    test "binary" do
      assert_from_term_same <<1, 2, 3>>
    end

    test "charlist" do
      assert_from_term_same 'testing some text'
    end

    test "list" do
      assert_from_term_same [:value, 1, true, "testing"]
    end

    test "keylist" do
      assert_from_term_same test: :value, more: "tests"
    end

    test "large_number" do
      assert_from_term_same 100_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000
    end

    test "nil" do
      assert_from_term_same nil
    end

    test "empty tuple" do
      assert_from_term_same {}
    end

    test "[<<203>>]" do
      assert_from_term_same [<<203>>]
    end

    test "improper list" do
      assert_from_term_same [1 | 1]
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
        assert_from_term_same data
      end
    end
  end

  describe "from_binary" do
    test "string" do
      assert_to_term_same "testing some text"
    end

    test "nil" do
      assert_to_term_same nil
    end

    test "float" do
      assert_to_term_same 1.23
    end

    test "integer" do
      assert_to_term_same 150
    end

    test "atom" do
      assert_to_term_same :hello
    end

    test "not existing atom" do
      # :hello_hello_hello_hello atom
      data =
        <<131, 100, 0, 23, 104, 101, 108, 108, 111, 95, 104, 101, 108, 108, 111, 95, 104, 101,
          108, 108, 111, 95, 104, 101, 108, 108, 111>>

      assert "hello_hello_hello_hello" == EtfLoader.from_binary!(data)
    end

    test "not existing atom, unsafe" do
      # :hi_hi_hi_hi_hi_hi_hi atom
      # we cannot match on the atom because then the atom is already defined and would not be unsafe
      data =
        <<131, 100, 0, 20, 104, 105, 95, 104, 105, 95, 104, 105, 95, 104, 105, 95, 104, 105, 95,
          104, 105, 95, 104, 105>>

      assert result = EtfLoader.from_binary!(data, unsafe_atom: true)
      assert is_atom(result)
    end

    test "list" do
      assert_to_term_same [:value, 1, true, "testing"]
    end

    test "empty list" do
      assert_to_term_same []
    end

    test "tuple" do
      assert_to_term_same {:value, 1, true, "testing"}
    end

    test "map" do
      assert_to_term_same %{key: "test"}
    end

    test "improper" do
      assert_to_term_same [0 | 0]
    end

    test "improper leading list" do
      assert_to_term_same [[0] | 0]
    end

    test "improper starting list" do
      assert_to_term_same [0, 1, 2, 3 | 0]
    end

    test "func" do
      assert {:error, _} =
               make_ref()
               |> :erlang.term_to_binary()
               |> EtfLoader.from_binary!()
    end

    property "works for all terms" do
      check all data <- my_term() do
        assert_to_term_same data
      end
    end
  end
end
