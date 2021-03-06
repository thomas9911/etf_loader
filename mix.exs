defmodule EtfLoader.MixProject do
  use Mix.Project

  def project do
    [
      app: :etf_loader,
      version: "0.1.0",
      elixir: "~> 1.10",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger]
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:rustler, "~> 0.22"},
      {:stream_data, "~> 0.5", only: [:test, :dev]},
      {:benchee, ">= 0.0.0", only: [:dev]}
    ]
  end
end
