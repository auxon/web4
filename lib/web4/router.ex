defmodule Web4.Router do
  use Plug.Router

  plug :match
  plug :dispatch

  get "/load" do
    url = conn.params["url"]
    # In a real implementation, this would communicate with the Rust backend
    {:ok, content} = HTTPoison.get(url)
    summary = "This is a placeholder summary."

    conn
    |> put_resp_content_type("application/json")
    |> send_resp(200, Jason.encode!(%{content: content.body, summary: summary}))
  end

  match _ do
    send_resp(conn, 404, "Not found")
  end
end
