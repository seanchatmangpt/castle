defmodule CastlePaaS.Persistence do
  @moduledoc false

  def record(resource, attrs, tenant \\ nil) when is_atom(resource) and is_map(attrs) do
    changeset = Ash.Changeset.for_create(resource, :record, attrs)

    changeset =
      if Ash.Resource.Info.multitenancy_strategy(resource) do
        Ash.Changeset.set_tenant(changeset, tenant)
      else
        changeset
      end

    Ash.create(changeset)
  end
end

defmodule CastlePaaS.AdmissionProvider do
  @moduledoc "Explicit O -> O* admission provider boundary."
  @callback admit(map() | struct(), list(), map()) :: {:ok, map()} | {:error, term()}
end

defmodule CastlePaaS.AdmissionProvider.Refuse do
  @behaviour CastlePaaS.AdmissionProvider

  @impl true
  def admit(_subject, _observations, _semantic_bundle),
    do: {:error, :BLOCKED_ADMISSION_PROVIDER_NOT_CONFIGURED}
end

defmodule CastlePaaS.AdmissionWitness do
  @moduledoc false

  @digest_keys ["witness_digest", "policy_digest", "evidence_digest"]

  def verify(witness, subject, now_epoch_ms)
      when is_map(witness) and is_integer(now_epoch_ms) do
    subject_id = external_id(subject)

    with true <- get(witness, "admitted") == true,
         "ALIVE" <- get(witness, "standing"),
         ^subject_id <- get(witness, "subject"),
         authority when is_binary(authority) and authority != "" <- get(witness, "authority"),
         expires when is_integer(expires) and expires >= now_epoch_ms <-
           get(witness, "expires_at_epoch_ms"),
         true <- Enum.all?(@digest_keys, &(digest?(get(witness, &1)))) do
      {:ok, stringify(witness)}
    else
      false -> {:error, :REFUSED_INVALID_ADMISSION_WITNESS}
      nil -> {:error, :REFUSED_INCOMPLETE_ADMISSION_WITNESS}
      _ -> {:error, :REFUSED_ADMISSION_WITNESS_MISMATCH}
    end
  end

  def verify(_, _, _), do: {:error, :REFUSED_INVALID_ADMISSION_WITNESS}

  def external_id(%{external_id: value}) when is_binary(value), do: value
  def external_id(%{"external_id" => value}) when is_binary(value), do: value
  def external_id(%{id: value}), do: to_string(value)
  def external_id(%{"id" => value}), do: to_string(value)
  def external_id(value) when is_binary(value), do: value
  def external_id(_), do: nil

  defp digest?(value) when is_binary(value),
    do: byte_size(value) == 64 and String.match?(value, ~r/\A[0-9a-fA-F]{64}\z/)

  defp digest?(_), do: false

  defp get(map, key), do: Map.get(map, key, Map.get(map, String.to_atom(key)))

  defp stringify(value) when is_map(value),
    do: Map.new(value, fn {key, item} -> {to_string(key), stringify(item)} end)

  defp stringify(value) when is_list(value), do: Enum.map(value, &stringify/1)
  defp stringify(value), do: value
end

defmodule CastlePaaS.ReceiptVerifier do
  @moduledoc "Independent receipt replay/verification boundary."
  @callback verify(map()) :: {:ok, map()} | {:error, term()}
end

defmodule CastlePaaS.ReceiptVerifier.Refuse do
  @behaviour CastlePaaS.ReceiptVerifier

  @impl true
  def verify(_receipt), do: {:error, :BLOCKED_RECEIPT_VERIFIER_NOT_CONFIGURED}
end

defmodule CastlePaaS.Reactors.RegisterSubject do
  use Reactor

  input :tenant
  input :attrs

  step :persist_subject do
    argument :tenant, input(:tenant)
    argument :attrs, input(:attrs)

    run fn %{tenant: tenant, attrs: attrs}, _context ->
      CastlePaaS.Persistence.record(CastlePaaS.Subject, attrs, tenant)
    end
  end

  return :persist_subject
end

defmodule CastlePaaS.Reactors.AdmitSubject do
  use Reactor

  input :tenant
  input :subject
  input :observations
  input :now_epoch_ms
  input :admission_provider

  step :semantic_bundle do
    run fn _args, _context -> CastlePaaS.semantic_bundle() end
  end

  step :manufacture_witness do
    argument :subject, input(:subject)
    argument :observations, input(:observations)
    argument :provider, input(:admission_provider)
    argument :semantic_bundle, result(:semantic_bundle)

    run fn %{provider: provider} = args, _context ->
      provider.admit(args.subject, args.observations, args.semantic_bundle)
    end
  end

  step :verify_witness do
    argument :witness, result(:manufacture_witness)
    argument :subject, input(:subject)
    argument :now_epoch_ms, input(:now_epoch_ms)

    run fn args, _context ->
      CastlePaaS.AdmissionWitness.verify(args.witness, args.subject, args.now_epoch_ms)
    end
  end

  step :persist_admission do
    argument :tenant, input(:tenant)
    argument :subject, input(:subject)
    argument :witness, result(:verify_witness)

    run fn args, _context ->
      CastlePaaS.Persistence.record(
        CastlePaaS.Admission,
        %{
          external_id: "admission:#{CastlePaaS.AdmissionWitness.external_id(args.subject)}",
          label: "CASTLE O* admission",
          standing: :ALIVE,
          digest: args.witness["witness_digest"],
          metadata: args.witness
        },
        args.tenant
      )
    end
  end

  step :result do
    argument :witness, result(:verify_witness)
    argument :admission, result(:persist_admission)

    run fn args, _context -> {:ok, %{witness: args.witness, admission: args.admission}} end
  end

  return :result
end

defmodule CastlePaaS.Reactors.ConstructIntent do
  use Reactor

  input :tenant
  input :admission_result
  input :intent
  input :now_epoch_ms

  step :verify_admission do
    argument :admission_result, input(:admission_result)
    argument :intent, input(:intent)
    argument :now_epoch_ms, input(:now_epoch_ms)

    run fn %{admission_result: %{witness: witness}, intent: intent, now_epoch_ms: now}, _ ->
      subject = Map.get(intent, :subject) || Map.get(intent, "subject")
      CastlePaaS.AdmissionWitness.verify(witness, subject, now)
    end
  end

  step :persist_plan do
    argument :tenant, input(:tenant)
    argument :intent, input(:intent)

    run fn args, _context ->
      process = Map.get(args.intent, :process) || Map.get(args.intent, "process") || %{}
      digest = :crypto.hash(:sha256, Jason.encode!(process)) |> Base.encode16(case: :lower)

      CastlePaaS.Persistence.record(
        CastlePaaS.Plan,
        %{
          external_id: Map.get(process, :id) || Map.get(process, "id") || "plan:unidentified",
          label: "CASTLE inert plan",
          standing: :PARTIAL_ALIVE,
          digest: digest,
          metadata: %{process: process}
        },
        args.tenant
      )
    end
  end

  step :persist_intent do
    argument :tenant, input(:tenant)
    argument :intent, input(:intent)
    argument :witness, result(:verify_admission)
    argument :plan, result(:persist_plan)

    run fn args, _context ->
      subject = Map.get(args.intent, :subject) || Map.get(args.intent, "subject")
      digest = :crypto.hash(:sha256, Jason.encode!(args.intent)) |> Base.encode16(case: :lower)

      case CastlePaaS.Persistence.record(
             CastlePaaS.ExecutionIntent,
             %{
               external_id: "intent:#{subject}:#{digest}",
               label: "CASTLE inert execution intent",
               standing: :PARTIAL_ALIVE,
               digest: digest,
               metadata: %{admission_witness: args.witness, intent: args.intent}
             },
             args.tenant
           ) do
        {:ok, record} ->
          {:ok,
           %{
             record: record,
             runtime_intent: Map.put(args.intent, :o_star, args.witness),
             plan: args.plan
           }}

        error ->
          error
      end
    end
  end

  return :persist_intent
end

defmodule CastlePaaS.Reactors.ExecuteIntent do
  @moduledoc "The single Reactor allowed to cross the CASTLE BRCE DO boundary."
  use Reactor

  input :constructed_intent
  input :kernel
  input :now_epoch_ms

  step :manufacture_construct do
    async? false
    argument :constructed_intent, input(:constructed_intent)
    argument :kernel, input(:kernel)

    run fn args, _context ->
      args.kernel.manufacture(args.constructed_intent.runtime_intent)
    end
  end

  step :execute_brce do
    async? false
    argument :kernel, input(:kernel)
    argument :now_epoch_ms, input(:now_epoch_ms)
    argument :construct, result(:manufacture_construct)

    run fn args, _context ->
      digest = args.construct["construct_digest"]
      request = args.construct["runtime_request"]
      args.kernel.execute(request, digest, args.now_epoch_ms)
    end
  end

  return :execute_brce
end

defmodule CastlePaaS.Reactors.QualifyEvidence do
  use Reactor

  input :tenant
  input :evidence

  step :persist_evidence do
    argument :tenant, input(:tenant)
    argument :evidence, input(:evidence)

    run fn %{tenant: tenant, evidence: evidence}, _context ->
      digest = Map.get(evidence, :digest) || Map.get(evidence, "digest")
      standing = Map.get(evidence, :standing) || Map.get(evidence, "standing") || :UNKNOWN

      CastlePaaS.Persistence.record(
        CastlePaaS.Evidence,
        %{
          external_id: Map.get(evidence, :external_id) || Map.get(evidence, "external_id") || "evidence:#{digest}",
          label: Map.get(evidence, :label) || Map.get(evidence, "label") || "CASTLE evidence",
          standing: normalize_standing(standing),
          digest: digest,
          metadata: evidence
        },
        tenant
      )
    end
  end

  return :persist_evidence

  defp normalize_standing(value) when is_atom(value), do: value
  defp normalize_standing(value) when is_binary(value), do: String.to_existing_atom(value)
end

defmodule CastlePaaS.Reactors.ReplayReceipt do
  use Reactor

  input :tenant
  input :receipt
  input :verifier

  step :verify_receipt do
    argument :receipt, input(:receipt)
    argument :verifier, input(:verifier)
    run fn args, _context -> args.verifier.verify(args.receipt) end
  end

  step :persist_replay do
    argument :tenant, input(:tenant)
    argument :verification, result(:verify_receipt)

    run fn args, _context ->
      digest = Map.get(args.verification, :receipt_digest) || Map.get(args.verification, "receipt_digest")

      CastlePaaS.Persistence.record(
        CastlePaaS.Replay,
        %{
          external_id: "replay:#{digest}",
          label: "CASTLE receipt replay",
          standing: :ALIVE,
          digest: digest,
          metadata: args.verification
        },
        args.tenant
      )
    end
  end

  return :persist_replay
end

defmodule CastlePaaS.Reactors.PublishProjection do
  @moduledoc "CONSTRUCT-only AshR2RML path/content manufacture; ggen owns filesystem DO."
  use Reactor

  step :compile_semantic_bundle do
    run fn _args, _context -> CastlePaaS.semantic_bundle() end
  end

  return :compile_semantic_bundle
end
