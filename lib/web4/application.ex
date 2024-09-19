defmodule Web4.Application do
  use Application

  def start(_type, _args) do
    children = [
      {Plug.Cowboy, scheme: :http, plug: Web4.Router, options: [port: 8000]}
    ]

    opts = [strategy: :one_for_one, name: Web4.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
