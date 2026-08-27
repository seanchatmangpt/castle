defmodule CastlePaaS.TestKernel do
  @behaviour CastlePaaS.Kernel

  @impl true
  def release_info, do: {:ok, %{"name" => "CASTLE", "release" => "test"}}

  @impl true
  def manufacture(intent) do
    digest = CastlePaaS.Canonical.sha256(intent)

    {:ok,
     %{
       "standing" => "ALIVE",
       "construct_digest" => digest,
       "construct_receipt_digest" => String.duplicate("a", 64),
       "runtime_request" => intent
     }}
  end

  @impl true
  def execute(request, digest, now_epoch_ms) do
    if digest == CastlePaaS.Canonical.sha256(request) and is_integer(now_epoch_ms) do
      {:ok,
       %{
         "standing" => "ALIVE",
         "brce_prepare_receipt_digests" => [String.duplicate("b", 64)],
         "brce_outcome_receipt_digests" => [String.duplicate("c", 64)],
         "evidence_commit" => %{"standing" => "ALIVE"}
       }}
    else
      {:error, :REFUSED_TEST_CONSTRUCT_MISMATCH}
    end
  end
end

defmodule CastlePaaS.PlatformContractTest do
  use ExUnit.Case, async: true

  test "generated Ash domain contains the complete 11-resource platform plane" do
    resources = Ash.Domain.Info.resources(CastlePaaS.Domain)

    assert length(resources) == 11

    assert MapSet.new(resources) ==
             MapSet.new([
               CastlePaaS.Organization,
               CastlePaaS.PlatformService,
               CastlePaaS.Subject,
               CastlePaaS.Observation,
               CastlePaaS.Admission,
               CastlePaaS.Plan,
               CastlePaaS.ExecutionIntent,
               CastlePaaS.Evidence,
               CastlePaaS.Receipt,
               CastlePaaS.Replay,
               CastlePaaS.Capability
             ])
  end

  test "AshR2RML manufactures a deterministic public semantic bundle without private fields" do
    assert {:ok, first} = CastlePaaS.semantic_bundle()
    assert {:ok, second} = CastlePaaS.semantic_bundle()
    assert first == second
    assert first.status == :PARTIAL_ALIVE
    assert first.standing == :construct_only
    assert is_map(first.files)
    assert map_size(first.files) >= 3

    semantic_text = first.files |> Map.values() |> Enum.filter(&is_binary/1) |> Enum.join("\n")

    refute semantic_text =~ "tenant_id"
    refute semantic_text =~ "metadata"
    refute semantic_text =~ "adapter_policy"
    refute semantic_text =~ "signing_key"

    assert semantic_text =~ "http://www.w3.org/ns/prov#"
    assert semantic_text =~ "http://www.w3.org/ns/sosa/Observation"
    assert semantic_text =~ "http://www.w3.org/ns/odrl/2/Request"
  end

  test "canonical identity is invariant to map insertion order" do
    left = %{subject: "system:one", process: %{id: "p1", values: [3, 2, 1]}, admitted: true}
    right = %{"admitted" => true, "process" => %{"values" => [3, 2, 1], "id" => "p1"}, "subject" => "system:one"}

    assert CastlePaaS.Canonical.sha256(left) == CastlePaaS.Canonical.sha256(right)
  end

  test "O star witness is exact-subject, digest-bound and expiring" do
    digest = String.duplicate("d", 64)

    witness = %{
      "admitted" => true,
      "standing" => "ALIVE",
      "subject" => "system:one",
      "authority" => "bounded-do",
      "expires_at_epoch_ms" => 1_000,
      "witness_digest" => digest,
      "policy_digest" => digest,
      "evidence_digest" => digest
    }

    assert {:ok, ^witness} = CastlePaaS.AdmissionWitness.verify(witness, "system:one", 999)
    assert {:error, :REFUSED_ADMISSION_WITNESS_MISMATCH} =
             CastlePaaS.AdmissionWitness.verify(witness, "system:other", 999)

    assert {:error, :REFUSED_ADMISSION_WITNESS_MISMATCH} =
             CastlePaaS.AdmissionWitness.verify(witness, "system:one", 1_001)
  end

  test "default admission and replay providers fail closed" do
    assert {:error, :BLOCKED_ADMISSION_PROVIDER_NOT_CONFIGURED} =
             CastlePaaS.AdmissionProvider.Refuse.admit(%{}, [], %{})

    assert {:error, :BLOCKED_RECEIPT_VERIFIER_NOT_CONFIGURED} =
             CastlePaaS.ReceiptVerifier.Refuse.verify(%{})
  end

  test "ExecuteIntent crosses DO only through construct then kernel execute" do
    runtime_intent = %{
      subject: "system:test",
      authority: "bounded-do",
      adapter_profile_id: "test",
      process: %{id: "powl:test", goal_id: "goal:test", activities: []},
      envelope: %{
        system_id: "system:test",
        allowed_transition_ids: [],
        max_steps: 0,
        expires_at_epoch_ms: 100
      },
      o_star: %{
        admitted: true,
        standing: "ALIVE",
        subject: "system:test",
        authority: "bounded-do"
      }
    }

    assert {:ok, result} =
             Reactor.run(
               CastlePaaS.Reactors.ExecuteIntent,
               %{
                 constructed_intent: %{runtime_intent: runtime_intent},
                 kernel: CastlePaaS.TestKernel,
                 now_epoch_ms: 50
               },
               %{},
               async?: false
             )

    assert result["standing"] == "ALIVE"
    assert length(result["brce_prepare_receipt_digests"]) == 1
    assert length(result["brce_outcome_receipt_digests"]) == 1
    assert result["evidence_commit"]["standing"] == "ALIVE"
  end

  test "typed standing parser cannot create arbitrary atoms" do
    assert {:ok, :ALIVE} = CastlePaaS.Standing.parse("ALIVE")
    assert {:error, _} = CastlePaaS.Standing.parse("NOT_A_CASTLE_STANDING")
  end
end
