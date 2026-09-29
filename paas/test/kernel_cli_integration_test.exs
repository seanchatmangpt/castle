defmodule CastlePaaS.KernelCLIIntegrationTest do
  use ExUnit.Case, async: false

  @moduletag :kernel_cli

  setup do
    profile = %{
      allowed_authorities: ["bounded-do"],
      adapter_policy: %{
        adapter_id: "paas-cli-proof",
        provider: "local",
        workload_identity: "workload:paas-cli-proof",
        commands: %{
          "echo" => %{
            transition_id: "echo",
            program: "/bin/echo",
            args: ["castle-paas-v26.8.26"],
            allowed_exit_codes: [0],
            max_output_bytes: 4096,
            timeout_ms: 2_000
          }
        }
      }
    }

    previous = Application.get_env(:castle_paas, :adapter_profiles)
    Application.put_env(:castle_paas, :adapter_profiles, %{"local-proof" => profile})

    on_exit(fn ->
      if is_nil(previous),
        do: Application.delete_env(:castle_paas, :adapter_profiles),
        else: Application.put_env(:castle_paas, :adapter_profiles, previous)
    end)

    :ok
  end

  test "fixed CLI port observes exact kernel identity and receipted BRCE DO" do
    digest = String.duplicate("d", 64)

    intent = %{
      adapter_profile_id: "local-proof",
      subject: "system:paas-cli-proof",
      authority: "bounded-do",
      o_star: %{
        admitted: true,
        standing: "ALIVE",
        subject: "system:paas-cli-proof",
        authority: "bounded-do",
        expires_at_epoch_ms: 10_000,
        witness_digest: digest,
        policy_digest: digest,
        evidence_digest: digest
      },
      config_graph: %{"zeroUnreceiptedActuation" => true},
      ontology: %{"version" => "26.8.18"},
      process: %{
        id: "powl:paas-cli-proof",
        goal_id: "goal:paas-cli-proof",
        activities: [
          %{id: "activity:echo", transition_id: "echo", predecessors: []}
        ]
      },
      envelope: %{
        system_id: "system:paas-cli-proof",
        allowed_transition_ids: ["echo"],
        max_steps: 1,
        expires_at_epoch_ms: 10_000
      }
    }

    assert {:ok, release} = CastlePaaS.Kernel.CLI.release_info()
    assert release["name"] == "CASTLE"
    assert byte_size(release["binary_sha256"]) == 64

    assert {:ok, construct} = CastlePaaS.Kernel.CLI.manufacture(intent)
    assert construct["standing"] == "ALIVE"
    assert byte_size(construct["construct_digest"]) == 64

    assert {:ok, executed} =
             CastlePaaS.Kernel.CLI.execute(
               construct["runtime_request"],
               construct["construct_digest"],
               2
             )

    assert executed["standing"] == "ALIVE"
    assert length(executed["brce_prepare_receipt_digests"]) == 1
    assert length(executed["brce_outcome_receipt_digests"]) == 1
    assert executed["evidence_commit"]["standing"] == "ALIVE"
  end
end
