defmodule EtfLoader.Error do
  defexception [:message, :type, :value]

  @impl true
  def exception(opts) do
    type = Keyword.get(opts, :type)
    value = Keyword.get(opts, :value)

    msg =
      if is_nil(type) do
        "cannot format term"
      else
        "cannot format term of type: '#{type}'"
      end

    %EtfLoader.Error{message: msg, type: type, value: value}
  end
end
