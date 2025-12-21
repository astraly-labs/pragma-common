use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use starknet_rust::{
    core::types::{
        BlockHashAndNumber, BlockId, BroadcastedDeclareTransaction,
        BroadcastedDeployAccountTransaction, BroadcastedInvokeTransaction, BroadcastedTransaction,
        ConfirmedBlockId, ContractClass, ContractStorageKeys, DeclareTransactionResult,
        DeployAccountTransactionResult, EventFilter, EventsPage, FeeEstimate, Felt, FunctionCall,
        Hash256, InvokeTransactionResult, MaybePreConfirmedBlockWithReceipts,
        MaybePreConfirmedBlockWithTxHashes, MaybePreConfirmedBlockWithTxs,
        MaybePreConfirmedStateUpdate, MessageFeeEstimate, MessageStatus, MsgFromL1,
        SimulatedTransaction, SimulationFlag, SimulationFlagForEstimateFee, StorageProof,
        SyncStatusType, Transaction, TransactionReceiptWithBlockInfo, TransactionStatus,
        TransactionTrace, TransactionTraceWithHash,
    },
    providers::{
        jsonrpc::HttpTransport, JsonRpcClient, Provider, ProviderError, ProviderRequestData,
        ProviderResponseData, Url,
    },
};
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};

/// Target status for waiting on transaction finality
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitForTarget {
    /// Wait for transaction to be accepted on L2
    AcceptedOnL2,
    /// Wait for transaction to be accepted on L1
    AcceptedOnL1,
}

/// A provider that automatically falls back to other RPC endpoints when the primary fails.
///
/// # Example
/// ```no_run
/// use starknet_providers::{FallbackProvider, jsonrpc::{HttpTransport, JsonRpcClient}};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create with a list of RPC URLs
/// let provider = FallbackProvider::new(vec![
///     "[https://primary-rpc.example.com](https://primary-rpc.example.com)",
///     "[https://secondary-rpc.example.com](https://secondary-rpc.example.com)",
///     "[https://tertiary-rpc.example.com](https://tertiary-rpc.example.com)",
/// ])?;
///
/// // Or create from existing clients
/// let clients = vec![
///     JsonRpcClient::new(HttpTransport::new("[https://primary-rpc.example.com](https://primary-rpc.example.com)")),
///     JsonRpcClient::new(HttpTransport::new("[https://secondary-rpc.example.com](https://secondary-rpc.example.com)")),
/// ];
/// let provider = FallbackProvider::from_clients(clients);
///
/// // Use it like any other provider
/// let block_number = provider.block_number().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct FallbackProvider {
    /// List of providers sorted by priority (index 0 = highest priority)
    providers: Vec<JsonRpcClient<HttpTransport>>,
    /// Current active provider index
    current_index: Arc<RwLock<usize>>,
    /// Whether to rotate through providers on error or always start from the first
    sticky_failover: bool,
}

impl FallbackProvider {
    /// Creates a new fallback provider from a list of RPC URLs.
    ///
    /// The URLs are used in order of priority (first URL = highest priority).
    pub fn new(urls: Vec<impl Into<Url>>) -> Result<Self, ProviderError> {
        let providers = urls
            .into_iter()
            .map(|url| JsonRpcClient::new(HttpTransport::new(url.into())))
            .collect();

        Ok(Self {
            providers,
            current_index: Arc::new(RwLock::new(0)),
            sticky_failover: false,
        })
    }

    /// Creates a new fallback provider from existing JSON-RPC clients.
    pub fn from_clients(clients: Vec<JsonRpcClient<HttpTransport>>) -> Self {
        if clients.is_empty() {
            panic!("FallbackProvider requires at least one client");
        }

        Self {
            providers: clients,
            current_index: Arc::new(RwLock::new(0)),
            sticky_failover: false,
        }
    }

    /// Enables sticky failover mode.
    ///
    /// When enabled, the provider will stick to a working provider instead of
    /// always trying the primary first. This can reduce latency when the primary
    /// is down for extended periods.
    pub fn with_sticky_failover(mut self, sticky: bool) -> Self {
        self.sticky_failover = sticky;
        self
    }

    /// Gets the number of available providers.
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    /// Gets the current active provider index.
    pub async fn current_provider_index(&self) -> usize {
        *self.current_index.read().await
    }

    /// Resets to use the primary provider.
    pub async fn reset_to_primary(&self) {
        *self.current_index.write().await = 0;
    }

    /// Waits for a transaction to reach the specified target status.
    pub async fn wait_for<H>(
        &self,
        transaction_hash: H,
        target: WaitForTarget,
        check_interval: Option<Duration>,
        timeout_duration: Option<Duration>,
    ) -> Result<(), ProviderError>
    where
        H: AsRef<Felt> + Send + Sync + Copy,
    {
        let check_interval = check_interval.unwrap_or(Duration::from_secs(10));
        let timeout_duration = timeout_duration.unwrap_or(Duration::from_secs(30 * 60));

        let tx_hash = *transaction_hash.as_ref();

        let wait_future = async {
            loop {
                match self.get_transaction_status(tx_hash).await {
                    Ok(status) => {
                        match status {
                            TransactionStatus::AcceptedOnL1(_) => {
                                // L1 acceptance means L2 is also satisfied
                                return Ok(());
                            }
                            TransactionStatus::AcceptedOnL2(_) => {
                                // If we're waiting for L2, we're done
                                // If we're waiting for L1, continue polling
                                match target {
                                    WaitForTarget::AcceptedOnL2 => return Ok(()),
                                    WaitForTarget::AcceptedOnL1 => {
                                        sleep(check_interval).await;
                                    }
                                }
                            }
                            TransactionStatus::Received
                            | TransactionStatus::PreConfirmed(_)
                            | TransactionStatus::Candidate => {
                                sleep(check_interval).await;
                            }
                        }
                    }
                    Err(e) => match e {
                        ProviderError::RateLimited => {
                            sleep(check_interval).await;
                        }
                        _ if e
                            .to_string()
                            .contains("Unable to complete request at this time.") =>
                        {
                            sleep(check_interval).await;
                        }
                        _ => return Err(e),
                    },
                }
            }
        };

        timeout(timeout_duration, wait_future).await.map_err(|_| {
            ProviderError::StarknetError(
                starknet_rust::core::types::StarknetError::UnexpectedError(format!(
                    "Timeout waiting for transaction {tx_hash:#x} to be accepted on {target:?} after {timeout_duration:?}",
                )),
            )
        })?
    }

    async fn execute_with_fallback<T, F>(&self, mut f: F) -> Result<T, ProviderError>
    where
        for<'a> F: FnMut(
            &'a JsonRpcClient<HttpTransport>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T, ProviderError>> + Send + 'a>,
        >,
    {
        let start_index = if self.sticky_failover {
            *self.current_index.read().await
        } else {
            0
        };

        let mut last_error = None;

        // Try each provider starting from the current/primary
        for offset in 0..self.providers.len() {
            let index = (start_index + offset) % self.providers.len();
            let provider = &self.providers[index];

            match f(provider).await {
                Ok(result) => {
                    // Update current index on success if using sticky failover
                    if self.sticky_failover && index != *self.current_index.read().await {
                        *self.current_index.write().await = index;
                    }
                    return Ok(result);
                }
                Err(err) => {
                    match err {
                        // If we're rate limited, we try a new provider
                        ProviderError::RateLimited => {
                            last_error = Some(err);
                            continue;
                        }
                        ProviderError::Other(err)
                            if err
                                .to_string()
                                .contains("Unable to complete request at this time.") =>
                        {
                            last_error = Some(ProviderError::Other(err));
                            continue;
                        }
                        ProviderError::Other(err)
                            if err.to_string().contains("error sending request") =>
                        {
                            last_error = Some(ProviderError::Other(err));
                            continue;
                        }
                        // Else we just bubble up the error
                        err => {
                            return Err(err);
                        }
                    }
                    // Continue to next provider
                }
            }
        }

        // All providers failed, return the last error
        Err(last_error.unwrap()) // Safe unwrap
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Provider for FallbackProvider {
    async fn starknet_version<B>(&self, block_id: B) -> Result<String, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        self.execute_with_fallback(|provider| Box::pin(provider.starknet_version(owned_block_id)))
            .await
    }

    async fn spec_version(&self) -> Result<String, ProviderError> {
        self.execute_with_fallback(|provider| Box::pin(provider.spec_version()))
            .await
    }

    async fn get_block_with_tx_hashes<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePreConfirmedBlockWithTxHashes, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_block_with_tx_hashes(owned_block_id))
        })
        .await
    }

    async fn get_block_with_txs<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePreConfirmedBlockWithTxs, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_block_with_txs(owned_block_id))
        })
        .await
    }

    async fn get_block_with_receipts<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePreConfirmedBlockWithReceipts, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_block_with_receipts(owned_block_id))
        })
        .await
    }

    async fn get_state_update<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePreConfirmedStateUpdate, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_state_update(owned_block_id))
        })
        .await
    }

    async fn get_storage_at<A, K, B>(
        &self,
        contract_address: A,
        key: K,
        block_id: B,
    ) -> Result<Felt, ProviderError>
    where
        A: AsRef<Felt> + Send + Sync,
        K: AsRef<Felt> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        let owned_address = *contract_address.as_ref();
        let owned_key = *key.as_ref();
        let owned_block_id = *block_id.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_storage_at(owned_address, owned_key, owned_block_id))
        })
        .await
    }

    async fn get_messages_status(
        &self,
        transaction_hash: Hash256,
    ) -> Result<Vec<MessageStatus>, ProviderError> {
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_messages_status(transaction_hash))
        })
        .await
    }

    async fn get_transaction_status<H>(
        &self,
        transaction_hash: H,
    ) -> Result<TransactionStatus, ProviderError>
    where
        H: AsRef<Felt> + Send + Sync,
    {
        let owned_tx_hash = *transaction_hash.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_transaction_status(owned_tx_hash))
        })
        .await
    }

    async fn get_transaction_by_hash<H>(
        &self,
        transaction_hash: H,
    ) -> Result<Transaction, ProviderError>
    where
        H: AsRef<Felt> + Send + Sync,
    {
        let owned_tx_hash = *transaction_hash.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_transaction_by_hash(owned_tx_hash))
        })
        .await
    }

    async fn get_transaction_by_block_id_and_index<B>(
        &self,
        block_id: B,
        index: u64,
    ) -> Result<Transaction, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_transaction_by_block_id_and_index(owned_block_id, index))
        })
        .await
    }

    async fn get_transaction_receipt<H>(
        &self,
        transaction_hash: H,
    ) -> Result<TransactionReceiptWithBlockInfo, ProviderError>
    where
        H: AsRef<Felt> + Send + Sync,
    {
        let owned_tx_hash = *transaction_hash.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_transaction_receipt(owned_tx_hash))
        })
        .await
    }

    async fn get_class<B, H>(
        &self,
        block_id: B,
        class_hash: H,
    ) -> Result<ContractClass, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        H: AsRef<Felt> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        let owned_class_hash = *class_hash.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_class(owned_block_id, owned_class_hash))
        })
        .await
    }

    async fn get_class_hash_at<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<Felt, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<Felt> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        let owned_address = *contract_address.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_class_hash_at(owned_block_id, owned_address))
        })
        .await
    }

    async fn get_class_at<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<ContractClass, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<Felt> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        let owned_address = *contract_address.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_class_at(owned_block_id, owned_address))
        })
        .await
    }

    async fn get_block_transaction_count<B>(&self, block_id: B) -> Result<u64, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_block_transaction_count(owned_block_id))
        })
        .await
    }

    async fn call<R, B>(&self, request: R, block_id: B) -> Result<Vec<Felt>, ProviderError>
    where
        R: AsRef<FunctionCall> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        let owned_request = request.as_ref().clone();
        let owned_block_id = *block_id.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.call(owned_request.clone(), owned_block_id))
        })
        .await
    }

    async fn estimate_fee<R, S, B>(
        &self,
        request: R,
        simulation_flags: S,
        block_id: B,
    ) -> Result<Vec<FeeEstimate>, ProviderError>
    where
        R: AsRef<[BroadcastedTransaction]> + Send + Sync,
        S: AsRef<[SimulationFlagForEstimateFee]> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        let owned_request = request.as_ref().to_vec();
        let owned_flags = simulation_flags.as_ref().to_vec();
        let owned_block_id = *block_id.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.estimate_fee(
                owned_request.clone(),
                owned_flags.clone(),
                owned_block_id,
            ))
        })
        .await
    }

    async fn estimate_message_fee<M, B>(
        &self,
        message: M,
        block_id: B,
    ) -> Result<MessageFeeEstimate, ProviderError>
    where
        M: AsRef<MsgFromL1> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        let owned_message = message.as_ref().clone();
        let owned_block_id = *block_id.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.estimate_message_fee(owned_message.clone(), owned_block_id))
        })
        .await
    }

    async fn block_number(&self) -> Result<u64, ProviderError> {
        self.execute_with_fallback(|provider| Box::pin(provider.block_number()))
            .await
    }

    async fn block_hash_and_number(&self) -> Result<BlockHashAndNumber, ProviderError> {
        self.execute_with_fallback(|provider| Box::pin(provider.block_hash_and_number()))
            .await
    }

    async fn chain_id(&self) -> Result<Felt, ProviderError> {
        self.execute_with_fallback(|provider| Box::pin(provider.chain_id()))
            .await
    }

    async fn syncing(&self) -> Result<SyncStatusType, ProviderError> {
        self.execute_with_fallback(|provider| Box::pin(provider.syncing()))
            .await
    }

    async fn get_events(
        &self,
        filter: EventFilter,
        continuation_token: Option<String>,
        chunk_size: u64,
    ) -> Result<EventsPage, ProviderError> {
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_events(filter.clone(), continuation_token.clone(), chunk_size))
        })
        .await
    }

    async fn get_nonce<B, A>(&self, block_id: B, contract_address: A) -> Result<Felt, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<Felt> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        let owned_address = *contract_address.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_nonce(owned_block_id, owned_address))
        })
        .await
    }

    async fn get_storage_proof<B, H, A, K>(
        &self,
        block_id: B,
        class_hashes: H,
        contract_addresses: A,
        contracts_storage_keys: K,
    ) -> Result<StorageProof, ProviderError>
    where
        B: AsRef<ConfirmedBlockId> + Send + Sync,
        H: AsRef<[Felt]> + Send + Sync,
        A: AsRef<[Felt]> + Send + Sync,
        K: AsRef<[ContractStorageKeys]> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        let owned_class_hashes = class_hashes.as_ref().to_vec();
        let owned_addresses = contract_addresses.as_ref().to_vec();
        let owned_keys = contracts_storage_keys.as_ref().to_vec();

        self.execute_with_fallback(move |provider| {
            Box::pin(provider.get_storage_proof(
                owned_block_id,
                owned_class_hashes.clone(),
                owned_addresses.clone(),
                owned_keys.clone(),
            ))
        })
        .await
    }

    async fn add_invoke_transaction<I>(
        &self,
        invoke_transaction: I,
    ) -> Result<InvokeTransactionResult, ProviderError>
    where
        I: AsRef<BroadcastedInvokeTransaction> + Send + Sync,
    {
        let owned_tx = invoke_transaction.as_ref().clone();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.add_invoke_transaction(owned_tx.clone()))
        })
        .await
    }

    async fn add_declare_transaction<D>(
        &self,
        declare_transaction: D,
    ) -> Result<DeclareTransactionResult, ProviderError>
    where
        D: AsRef<BroadcastedDeclareTransaction> + Send + Sync,
    {
        let owned_tx = declare_transaction.as_ref().clone();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.add_declare_transaction(owned_tx.clone()))
        })
        .await
    }

    async fn add_deploy_account_transaction<D>(
        &self,
        deploy_account_transaction: D,
    ) -> Result<DeployAccountTransactionResult, ProviderError>
    where
        D: AsRef<BroadcastedDeployAccountTransaction> + Send + Sync,
    {
        let owned_tx = deploy_account_transaction.as_ref().clone();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.add_deploy_account_transaction(owned_tx.clone()))
        })
        .await
    }

    async fn trace_transaction<H>(
        &self,
        transaction_hash: H,
    ) -> Result<TransactionTrace, ProviderError>
    where
        H: AsRef<Felt> + Send + Sync,
    {
        let owned_tx_hash = *transaction_hash.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.trace_transaction(owned_tx_hash))
        })
        .await
    }

    async fn simulate_transactions<B, T, S>(
        &self,
        block_id: B,
        transactions: T,
        simulation_flags: S,
    ) -> Result<Vec<SimulatedTransaction>, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        T: AsRef<[BroadcastedTransaction]> + Send + Sync,
        S: AsRef<[SimulationFlag]> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        let owned_txs = transactions.as_ref().to_vec();
        let owned_flags = simulation_flags.as_ref().to_vec();

        self.execute_with_fallback(move |provider| {
            Box::pin(provider.simulate_transactions(
                owned_block_id,
                owned_txs.clone(),
                owned_flags.clone(),
            ))
        })
        .await
    }

    async fn trace_block_transactions<B>(
        &self,
        block_id: B,
    ) -> Result<Vec<TransactionTraceWithHash>, ProviderError>
    where
        B: AsRef<ConfirmedBlockId> + Send + Sync,
    {
        let owned_block_id = *block_id.as_ref();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.trace_block_transactions(owned_block_id))
        })
        .await
    }

    async fn batch_requests<R>(
        &self,
        requests: R,
    ) -> Result<Vec<ProviderResponseData>, ProviderError>
    where
        R: AsRef<[ProviderRequestData]> + Send + Sync,
    {
        let owned_requests = requests.as_ref().to_vec();
        self.execute_with_fallback(move |provider| {
            Box::pin(provider.batch_requests(owned_requests.clone()))
        })
        .await
    }
}
