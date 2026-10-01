defmodule CastlePaaS do
  @moduledoc """
  Ash application/control plane for CASTLE.

  Ash owns domain actions and persistence, AshR2RML owns semantic correspondence,
  Reactor owns saga orchestration, ggen owns generated filesystem consequences, and
  the CASTLE kernel remains the exclusive admission/BRCE authority boundary.
  """

  @spec semantic_bundle() :: {:ok, map()} | {:error, term()}
  def semantic_bundle do
    CastlePaaS.Domain
    |> Ash.Domain.Info.resources()
    |> AshR2RML.Ggen.compile_ash_ttl_bundle()
  end

  @spec kernel() :: module()
  def kernel, do: Application.fetch_env!(:castle_paas, :kernel_module)

  @spec admission_provider() :: module()
  def admission_provider, do: Application.fetch_env!(:castle_paas, :admission_provider)

  @spec receipt_verifier() :: module()
  def receipt_verifier, do: Application.fetch_env!(:castle_paas, :receipt_verifier)
end

defmodule CastlePaaS.Repo do
  use AshPostgres.Repo, otp_app: :castle_paas

  @impl AshPostgres.Repo
  def installed_extensions, do: ["ash-functions"]

  @impl AshPostgres.Repo
  def min_pg_version, do: %Version{major: 14, minor: 0, patch: 0}
end

defmodule CastlePaaS.Application do
  use Application

  @impl true
  def start(_type, _args) do
    Supervisor.start_link(
      [CastlePaaS.Repo],
      strategy: :one_for_one,
      name: CastlePaaS.Supervisor
    )
  end
end
