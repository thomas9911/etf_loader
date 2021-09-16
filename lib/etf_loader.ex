defmodule EtfLoader do
  @moduledoc """


  Bitstring do not work in nifs:

  Quote from erl_nif: 
  'Bitstrings with an arbitrary bit length have no support yet.'
  from `https://erlang.org/doc/man/erl_nif.html`
  """
  use Rustler, otp_app: :etf_loader

  # When your NIF is loaded, it will override this function.
  def to_binary(_a), do: :erlang.nif_error(:nif_not_loaded)

  def to_binary!(term) do
    {:ok, binary} = to_binary(term)
    binary
  end

  def from_binary(input), do: from_binary(input, [])
  def from_binary(_a, _b), do: :erlang.nif_error(:nif_not_loaded)

  def from_binary!(term, opts \\ []) do
    {:ok, term} = from_binary(term, opts)
    term
  end
end
