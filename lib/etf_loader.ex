defmodule EtfLoader do
  @moduledoc """


  Bitstring do not work in nifs:

  Quote from erl_nif: 
  'Bitstrings with an arbitrary bit length have no support yet.'
  from `https://erlang.org/doc/man/erl_nif.html`
  """
  use Rustler, otp_app: :etf_loader

  # copied from List.improper?
  defguardp is_proper_list(list) when is_list(list) and length(list) >= 0

  def to_binary(_a), do: :erlang.nif_error(:nif_not_loaded)

  def to_binary!(term) do
    {:ok, binary} = to_binary(term)
    binary
  end

  def from_binary(input, opts \\ []) do
    case internal_from_binary(input, opts) do
      {:ok, term} ->
        {:ok, convert_term_to_term(term)}

      term ->
        term
    end
  end

  # We cant use native large integers in nif, so we encoded it as a list
  defp convert_term_to_term({:__etf_loader_big_int, list_integer}) do
    convert_term_to_term(:erlang.list_to_integer(list_integer))
  end

  defp convert_term_to_term(tuple) when is_tuple(tuple) do
    tuple
    |> Tuple.to_list()
    |> convert_term_to_term()
    |> List.to_tuple()
  end

  defp convert_term_to_term(map) when is_map(map) and not is_struct(map) do
    Map.new(map, fn {key, value} -> {convert_term_to_term(key), convert_term_to_term(value)} end)
  end

  defp convert_term_to_term(list) when is_proper_list(list) do
    Enum.map(list, &convert_term_to_term/1)
  end

  defp convert_term_to_term([a | b]) do
    [convert_term_to_term(a) | convert_term_to_term(b)]
  end

  defp convert_term_to_term(term), do: term

  defp internal_from_binary(_a, _b), do: :erlang.nif_error(:nif_not_loaded)

  def from_binary!(term, opts \\ []) do
    {:ok, term} = from_binary(term, opts)
    term
  end
end
