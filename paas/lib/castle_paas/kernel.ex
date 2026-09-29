defmodule CastlePaaS.Kernel do
  @moduledoc """
  Narrow port to the CASTLE Rust kernel.

  No callback accepts shell command text. The CLI implementation maps operations to
  fixed observed CASTLE nouns/verbs and injects provider command policy only from a
  server-side adapter profile selected by id.
  """

  @type result :: {:ok, map()} | {:error, term()}

  @callback release_info() :: result()
  @callback manufacture(map()) :: result()
  @callback execute(map(), String.t(), integer()) :: result()
end

defmodule CastlePaaS.Kernel.CLI do
  @moduledoc """
  Fixed-verb CASTLE CLI adapter.

  Required runtime configuration:

    * `CASTLE_BIN` - exact CASTLE binary path
    * `CASTLE_BIN_SHA256` - expected binary SHA-256
    * `CASTLE_SIGNING_KEY_PATH` - server-side Ed25519 seed file path
    * `CASTLE_KEY_ID` - receipt key identifier
    * `:castle_paas, :adapter_profiles` - allowlisted provider policies keyed by id

  Client/model input can select an admitted profile id but cannot supply a program,
  argv, key path, or provider command map.
  """

  @behaviour CastlePaaS.Kernel

  @impl true
  def release_info do
    with {:ok, runtime} <- runtime(),
         {:ok, result} <- run(runtime.bin, ["release", "info", "--format", "json"]),
         :ok <- require_castle_identity(result) do
      {:ok, Map.put(result, "binary_sha256", runtime.bin_sha256)}
    end
  end

  @impl true
  def manufacture(intent) when is_map(intent) do
    with {:ok, runtime} <- runtime(),
         {:ok, request} <- build_request(intent),
         {:ok, request_path} <- write_request(request),
         result <-
           run(runtime.bin, [
             "construct",
             "manufacture",
             "--request-path",
             request_path,
             "--signing-key-path",
             runtime.signing_key_path,
             "--key-id",
             runtime.key_id,
             "--format",
             "json"
           ]),
         :ok <- File.rm(request_path),
         {:ok, summary} <- result,
         :ok <- require_alive_construct(summary) do
      {:ok,
       summary
       |> Map.put("kernel_binary_sha256", runtime.bin_sha256)
       |> Map.put("runtime_request", request)}
    else
      {:error, _} = error -> error
      other -> {:error, {:REFUSED_KERNEL_MANUFACTURE, other}}
    end
  end

  @impl true
  def execute(request, expected_construct_digest, now_epoch_ms)
      when is_map(request) and is_binary(expected_construct_digest) and is_integer(now_epoch_ms) do
    with {:ok, runtime} <- runtime(),
         :ok <- digest?(expected_construct_digest),
         {:ok, request_path} <- write_request(request),
         result <-
           run(runtime.bin, [
             "do",
             "execute",
             "--request-path",
             request_path,
             "--signing-key-path",
             runtime.signing_key_path,
             "--key-id",
             runtime.key_id,
             "--expected-construct-digest",
             expected_construct_digest,
             "--now-epoch-ms",
             Integer.to_string(now_epoch_ms),
             "--format",
             "json"
           ]),
         :ok <- File.rm(request_path),
         {:ok, summary} <- result,
         :ok <- require_receipted_do(summary) do
      {:ok, Map.put(summary, "kernel_binary_sha256", runtime.bin_sha256)}
    else
      {:error, _} = error -> error
      other -> {:error, {:REFUSED_KERNEL_DO, other}}
    end
  end

  defp runtime do
    with bin when is_binary(bin) and bin != "" <- System.get_env("CASTLE_BIN"),
         expected when is_binary(expected) and byte_size(expected) == 64 <-
           System.get_env("CASTLE_BIN_SHA256"),
         signing_key_path when is_binary(signing_key_path) and signing_key_path != "" <-
           System.get_env("CASTLE_SIGNING_KEY_PATH"),
         key_id when is_binary(key_id) and key_id != "" <- System.get_env("CASTLE_KEY_ID"),
         {:ok, bytes} <- File.read(bin),
         actual <- Base.encode16(:crypto.hash(:sha256, bytes), case: :lower),
         true <- actual == String.downcase(expected),
         true <- File.regular?(signing_key_path) do
      {:ok,
       %{
         bin: bin,
         bin_sha256: actual,
         signing_key_path: signing_key_path,
         key_id: key_id
       }}
    else
      nil -> {:error, :BLOCKED_CASTLE_RUNTIME_CONFIGURATION}
      false -> {:error, :REFUSED_CASTLE_RUNTIME_IDENTITY}
      {:error, reason} -> {:error, {:BLOCKED_CASTLE_BINARY, reason}}
      _ -> {:error, :REFUSED_CASTLE_RUNTIME_CONFIGURATION}
    end
  end

  defp build_request(intent) do
    forbidden = ["adapter_policy", :adapter_policy, "commands", :commands, "program", :program]

    if Enum.any?(forbidden, &Map.has_key?(intent, &1)) do
      {:error, :REFUSED_AMBIENT_COMMAND_POLICY}
    else
      profile_id = Map.get(intent, :adapter_profile_id) || Map.get(intent, "adapter_profile_id")
      profiles = Application.get_env(:castle_paas, :adapter_profiles, %{})

      with profile when is_map(profile) <- Map.get(profiles, profile_id),
           {:ok, subject} <- required_string(intent, :subject),
           {:ok, authority} <- required_string(intent, :authority),
           {:ok, process} <- required_map(intent, :process),
           {:ok, envelope} <- required_map(intent, :envelope),
           {:ok, o_star} <- required_map(intent, :o_star),
           true <- Map.get(o_star, "admitted", Map.get(o_star, :admitted)) == true,
           adapter_policy when is_map(adapter_policy) <- Map.get(profile, :adapter_policy),
           allowed_authorities when is_list(allowed_authorities) <-
             Map.get(profile, :allowed_authorities) do
        evidence_dir =
          Path.join(
            System.tmp_dir!(),
            "castle-paas-evidence-#{System.unique_integer([:positive, :monotonic])}"
          )

        {:ok,
         %{
           "cell_id" => Map.get(intent, :cell_id, "cell:castle-paas"),
           "evidence_dir" => evidence_dir,
           "subject" => subject,
           "authority" => authority,
           "o_star" => stringify(o_star),
           "config_graph" =>
             stringify(Map.get(intent, :config_graph, %{"zeroUnreceiptedActuation" => true})),
           "ontology" => stringify(Map.get(intent, :ontology, %{"version" => "26.8.18"})),
           "process" => stringify(process),
           "envelope" => stringify(envelope),
           "allowed_authorities" => allowed_authorities,
           "adapter_policy" => stringify(adapter_policy)
         }}
      else
        nil -> {:error, :REFUSED_UNKNOWN_ADAPTER_PROFILE}
        false -> {:error, :REFUSED_O_STAR_REQUIRED}
        {:error, _} = error -> error
        _ -> {:error, :REFUSED_INVALID_ADAPTER_PROFILE}
      end
    end
  end

  defp write_request(request) do
    path =
      Path.join(
        System.tmp_dir!(),
        "castle-paas-request-#{System.unique_integer([:positive, :monotonic])}.json"
      )

    with {:ok, encoded} <- Jason.encode(request),
         :ok <- File.write(path, encoded, [:exclusive]) do
      {:ok, path}
    end
  end

  defp run(bin, args) do
    try do
      case System.cmd(bin, args, stderr_to_stdout: true) do
        {output, 0} ->
          case Jason.decode(output) do
            {:ok, value} when is_map(value) -> {:ok, value}
            _ -> {:error, {:REFUSED_NON_JSON_KERNEL_RESPONSE, output}}
          end

        {output, status} ->
          {:error, {:REFUSED_KERNEL_EXIT, status, output}}
      end
    rescue
      error -> {:error, {:BLOCKED_KERNEL_TRANSPORT, Exception.message(error)}}
    end
  end

  defp require_castle_identity(%{"name" => "CASTLE", "release" => release})
       when is_binary(release),
       do: :ok

  defp require_castle_identity(_), do: {:error, :REFUSED_WRONG_KERNEL_IDENTITY}

  defp require_alive_construct(%{"standing" => "ALIVE", "construct_digest" => digest}),
    do: digest?(digest)

  defp require_alive_construct(_), do: {:error, :REFUSED_CONSTRUCT_NOT_ALIVE}

  defp require_receipted_do(%{
         "standing" => "ALIVE",
         "brce_prepare_receipt_digests" => prepare,
         "brce_outcome_receipt_digests" => outcome,
         "evidence_commit" => %{"standing" => "ALIVE"}
       })
       when is_list(prepare) and is_list(outcome) and prepare != [] and outcome != [] do
    if Enum.all?(prepare ++ outcome, &(digest?(&1) == :ok)) do
      :ok
    else
      {:error, :REFUSED_INVALID_BRCE_RECEIPT_DIGEST}
    end
  end

  defp require_receipted_do(_), do: {:error, :REFUSED_UNRECEIPTED_DO}

  defp digest?(value)
       when is_binary(value) and byte_size(value) == 64 do
    if String.match?(value, ~r/\A[0-9a-fA-F]{64}\z/),
      do: :ok,
      else: {:error, :REFUSED_INVALID_DIGEST}
  end

  defp digest?(_), do: {:error, :REFUSED_INVALID_DIGEST}

  defp required_string(map, key) do
    value = Map.get(map, key) || Map.get(map, Atom.to_string(key))
    if is_binary(value) and value != "", do: {:ok, value}, else: {:error, {:REFUSED_REQUIRED_FIELD, key}}
  end

  defp required_map(map, key) do
    value = Map.get(map, key) || Map.get(map, Atom.to_string(key))
    if is_map(value), do: {:ok, value}, else: {:error, {:REFUSED_REQUIRED_FIELD, key}}
  end

  defp stringify(value) when is_map(value),
    do: Map.new(value, fn {key, item} -> {to_string(key), stringify(item)} end)

  defp stringify(value) when is_list(value), do: Enum.map(value, &stringify/1)
  defp stringify(value), do: value
end
