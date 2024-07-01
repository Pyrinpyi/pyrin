import asyncio
import unittest
import pyrin

class TestWallet(unittest.IsolatedAsyncioTestCase):

    @unittest.skip
    async def test_ping(self):
        rpc = pyrin.RPC()
        success = await rpc.connect() # TODO: endpoint as param and test gRPC as well, also stop with False at connect failed with timeout ?
        self.assertTrue(success)
        await rpc.ping()

    @unittest.skip
    async def test_get_metrics(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        result = await rpc.get_metrics(True, True, True, True)

        print("process_metrics:")
        print(result.process_metrics.resident_set_size) # 334454784
        print(result.process_metrics.virtual_memory_size) # 594845696
        print(result.process_metrics.core_num) # 20
        print(result.process_metrics.cpu_usage) # 0.28120216727256775
        print(result.process_metrics.fd_num) # 596
        print(result.process_metrics.disk_io_read_bytes) # 748633484
        print(result.process_metrics.disk_io_write_bytes) # 178008144
        print(result.process_metrics.disk_io_read_per_sec) # 559988.875
        print(result.process_metrics.disk_io_write_per_sec) # 20455.455078125

        print("connection_metrics:")
        print(result.connection_metrics.borsh_live_connections) # 1
        print(result.connection_metrics.borsh_connection_attempts) # 7
        print(result.connection_metrics.borsh_handshake_failures) # 0
        print(result.connection_metrics.json_live_connections) # 0
        print(result.connection_metrics.json_connection_attempts) # 0
        print(result.connection_metrics.json_handshake_failures) # 0
        print(result.connection_metrics.active_peers) # 8

        print("bandwidth_metrics:")
        print(result.bandwidth_metrics.borsh_bytes_tx) # 1511
        print(result.bandwidth_metrics.borsh_bytes_rx) # 104
        print(result.bandwidth_metrics.json_bytes_tx) # 0
        print(result.bandwidth_metrics.json_bytes_rx) # 0
        print(result.bandwidth_metrics.p2p_bytes_tx) # 346863
        print(result.bandwidth_metrics.p2p_bytes_rx) # 754849
        print(result.bandwidth_metrics.grpc_bytes_tx) # 0
        print(result.bandwidth_metrics.grpc_bytes_rx) # 0

        print("consensus_metrics:")
        print(result.consensus_metrics.node_blocks_submitted_count) # 516
        print(result.consensus_metrics.node_headers_processed_count) # 441
        print(result.consensus_metrics.node_dependencies_processed_count) # 688
        print(result.consensus_metrics.node_bodies_processed_count) # 441
        print(result.consensus_metrics.node_transactions_processed_count) # 475
        print(result.consensus_metrics.node_chain_blocks_processed_count) # 329
        print(result.consensus_metrics.node_mass_processed_count) # 170432
        print(result.consensus_metrics.node_database_blocks_count) # 225637
        print(result.consensus_metrics.node_database_headers_count) # 225637
        print(result.consensus_metrics.network_mempool_size) # 0
        print(result.consensus_metrics.network_tip_hashes_count) # 2
        print(result.consensus_metrics.network_difficulty) # 235332126575020.44
        print(result.consensus_metrics.network_past_median_time) # 1719761907837
        print(result.consensus_metrics.network_virtual_parent_hashes_count) # 2
        print(result.consensus_metrics.network_virtual_daa_score) # 18943188

        print("server_time:")
        print(result.server_time) # 1719762094334

    @unittest.skip
    async def test_get_server_info(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        result = await rpc.get_server_info()

        print(result.rpc_api_version) # [0, 1, 0, 0]
        print(result.server_version) # 0.14.1
        print(result.network_id) # mainnet
        print(result.has_utxo_index) # True
        print(result.is_synced) # True
        print(result.virtual_daa_score) # 18944668

    async def test_get_sync_status(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        self.assertTrue(await rpc.get_sync_status())

    # @unittest.skip
    async def test_get_current_network(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        self.assertEqual(await rpc.get_current_network(), "mainnet")

    @unittest.skip # TODO:
    async def test_submit_block(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        error = await rpc.submit_block({
            "header": {
                "hash": "bb149e176aecf79a60a22f57bc1c50b145100f7b6ec8bcb817070907a7ef44ec",
                "parents_by_level": [["bb149e176aecf79a60a22f57bc1c50b145100f7b6ec8bcb817070907a7ef44ec"]],
                "hash_merkle_root": "bb149e176aecf79a60a22f57bc1c50b145100f7b6ec8bcb817070907a7ef44ec",
                "accepted_id_merkle_root": "bb149e176aecf79a60a22f57bc1c50b145100f7b6ec8bcb817070907a7ef44ec",
                "utxo_commitment": "bb149e176aecf79a60a22f57bc1c50b145100f7b6ec8bcb817070907a7ef44ec",
                "version": 1,
                "timestamp": 1,
                "bits": 1,
                "nonce": 1,
                "daa_score": 1,
                "blue_work": 1,
                "blue_score": 1,
                "pruning_point": "bb149e176aecf79a60a22f57bc1c50b145100f7b6ec8bcb817070907a7ef44ec",
            },
            "transactions": [{
                "version": 1,
                "inputs": [{
                    "previous_outpoint": {
                        "transaction_id": "9e30e8d0327480c6c9c6b227537c018827a25e7e2e51280e8328acdbcf0fe76c",
                        "index": 0,
                    },
                    "signature_script": [],
                    "sequence": 0,
                    "sig_op_count": 0,
                    "verbose_data": None,
                }],
                "outputs": [],
                "lock_time": 1,
                "subnetwork_id": "0100000000000000000000000000000000000000",
                "gas": 1,
                "payload": [],
                "mass": 1,
                "verbose_data": {
                    "transaction_id": "bb149e176aecf79a60a22f57bc1c50b145100f7b6ec8bcb817070907a7ef44ec",
                    "hash": "bb149e176aecf79a60a22f57bc1c50b145100f7b6ec8bcb817070907a7ef44ec",
                    "mass": 1,
                    "block_hash": "bb149e176aecf79a60a22f57bc1c50b145100f7b6ec8bcb817070907a7ef44ec",
                    "block_time": 1,
                },
            }],

            # "hash": "bb149e176aecf79a60a22f57bc1c50b145100f7b6ec8bcb817070907a7ef44ec",
            # "transactions": 1,
            # "verbose_data": 1,
        }, True) # allow_non_daa_blocks

        if error > 0:
            # error == 1: block invalid
            # error == 2: is in ibd
            print(f"Failed to submit block: {error}")
        else:
            print("Block submitted successfully")

    @unittest.skip # TODO:
    async def test_get_block_template(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        result = await rpc.get_block_template("pyrin:qzn54t6vpasykvudztupcpwn2gelxf8y9p84szksr73me39mzf69uaalnymtx", [])
        print("result:", result.transactions[0].version)
        print("subnetwork_id:", result.transactions[0].subnetwork_id)
        print("verbose_data:", result.transactions[0].verbose_data)
        # print("result:", result.transactions[0].inputs[0].previous_outpoint.transaction_id)
        # print("result:", result.transactions[0].inputs[0].previous_outpoint.index)
        print("script_public_key:", result.transactions[0].outputs[0].script_public_key)
        print("result:", result.transactions[0].outputs[0].verbose_data)
        # print("result:", result.transactions[0].outputs[0].verbose_data.script_public_key_type) # TODO:
        # print("result:", result.transactions[0].outputs[0].verbose_data.script_public_key_address) # TODO:
        print("value:", result.transactions[0].outputs[0].value)

    @unittest.skip
    async def test_get_peer_addresses(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        result = await rpc.get_peer_addresses()
        print("result.known_addresses:", result.known_addresses) # ["127.0.0.1:13111"]
        print("result.banned_addresses:", result.banned_addresses) # ["127.0.0.1:13111"]

    @unittest.skip
    async def test_get_sink(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        result = await rpc.get_sink()
        print("result.sink:", result.sink) # 69c2ef342b17daddb222fe1cc94faab477f31b91f8f84496edfde430c41ca9cf

    @unittest.skip
    async def test_get_mempool_entry(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        result = await rpc.get_mempool_entry("a419045a31afad611c32344fa269e712499d3e97f74271e4a2deffa734ba9f71", True, True)
        print("result", result)

    @unittest.skip
    async def test_get_mempool_entries(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        result = await rpc.get_mempool_entries(True, True)
        print("get_mempool_entries", result)

    @unittest.skip
    async def test_get_connected_peer_info(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        result = await rpc.get_connected_peer_info()
        print("peer_info[0].id", result.peer_info[0].id) # sfdgabdb-4043-3e46-c333-adfasdfga6f4
        print("peer_info[0].address", result.peer_info[0].address) # 127.0.0.1:13111
        print("peer_info[0].last_ping_duration", result.peer_info[0].last_ping_duration) # 80
        print("peer_info[0].is_outbound", result.peer_info[0].is_outbound) # True
        print("peer_info[0].time_offset", result.peer_info[0].time_offset) # 677
        print("peer_info[0].user_agent", result.peer_info[0].user_agent) # /pyipad:0.13.4/pyipad:0.13.4/
        print("peer_info[0].advertised_protocol_version", result.peer_info[0].advertised_protocol_version) # 5
        print("peer_info[0].time_connected", result.peer_info[0].time_connected) # 24211336
        print("peer_info[0].is_ibd_peer", result.peer_info[0].is_ibd_peer) # False

    @unittest.skip
    async def test_add_peer(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        await rpc.add_peer("192.168.1.2:13111", True)

    @unittest.skip
    async def test_submit_transaction(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        await rpc.submit_transaction({
            "version": 0,
            "lock_time": 0,
            "subnetwork_id": "0100000000000000000000000000000000000000",
            "gas": 0,
            "payload": [],
            "mass": 0,
            # "inputs": [],
            "inputs": [{
                "previous_outpoint": {
                    "transaction_id": "9e30e8d0327480c6c9c6b227537c018827a25e7e2e51280e8328acdbcf0fe76c",
                    "index": 0,
                },
                "signature_script": [],
                "sequence": 0,
                "sig_op_count": 0,
                "verbose_data": None,
            }],
        }, True)

    @unittest.skip
    async def test_get_block(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        block = await rpc.get_block("f269afa4a29dd7e751e15343d9edbb132e2e2f7daecb71b29908c9f6922ec165", True)

        # Header
        print("block.header.hash:", block.header.hash)
        print("block.header.version:", block.header.version)
        print("block.header.parents_by_level:", block.header.parents_by_level)
        print("block.header.hash_merkle_root:", block.header.hash_merkle_root)
        print("block.header.accepted_id_merkle_root:", block.header.accepted_id_merkle_root)
        print("block.header.utxo_commitment:", block.header.utxo_commitment)
        print("block.header.timestamp:", block.header.timestamp)
        print("block.header.bits:", block.header.bits)
        print("block.header.nonce:", block.header.nonce)
        print("block.header.daa_score:", block.header.daa_score)
        print("block.header.blue_work:", block.header.blue_work)
        print("block.header.blue_score:", block.header.blue_score)
        print("block.header.pruning_point:", block.header.pruning_point)

        # Transactions
        print("block.transactions[0].version:", block.transactions[0].version)
        print("block.transactions[0].lock_time:", block.transactions[0].lock_time)
        print("block.transactions[0].subnetwork_id:", block.transactions[0].subnetwork_id)
        print("block.transactions[0].gas:", block.transactions[0].gas)
        print("block.transactions[0].payload:", block.transactions[0].payload)
        print("block.transactions[0].mass:", block.transactions[0].mass)
        print("\n")

        print("block.transactions[1].inputs[0].previous_outpoint.transaction_id:", block.transactions[1].inputs[0].previous_outpoint.transaction_id)
        print("block.transactions[1].inputs[0].previous_outpoint.index:", block.transactions[1].inputs[0].previous_outpoint.index)
        print("block.transactions[1].inputs[0].signature_script:", block.transactions[1].inputs[0].signature_script)
        print("block.transactions[1].inputs[0].sequence:", block.transactions[1].inputs[0].sequence)
        print("block.transactions[1].inputs[0].sig_op_count:", block.transactions[1].inputs[0].sig_op_count)
        print("block.transactions[1].inputs[0].verbose_data:", block.transactions[1].inputs[0].verbose_data)
        print("\n")

        print("block.transactions[0].outputs[0].script_public_key:", block.transactions[0].outputs[0].script_public_key)
        print("block.transactions[0].outputs[0].value:", block.transactions[0].outputs[0].value)
        print("block.transactions[0].outputs[0].verbose_data:", block.transactions[0].outputs[0].verbose_data.script_public_key_type)
        print("block.transactions[0].outputs[0].verbose_data:", block.transactions[0].outputs[0].verbose_data.script_public_key_address)

        # Verbose data
        print("block.verbose_data.difficulty:", block.verbose_data.difficulty)
        print("block.verbose_data.selected_parent_hash:", block.verbose_data.selected_parent_hash)
        print("block.verbose_data.transaction_ids:", block.verbose_data.transaction_ids)
        print("block.verbose_data.is_header_only:", block.verbose_data.is_header_only)
        print("block.verbose_data.blue_score:", block.verbose_data.blue_score)
        print("block.verbose_data.children_hashes:", block.verbose_data.children_hashes)
        print("block.verbose_data.merge_set_blues_hashes:", block.verbose_data.merge_set_blues_hashes)
        print("block.verbose_data.merge_set_reds_hashes:", block.verbose_data.merge_set_reds_hashes)
        print("block.verbose_data.is_chain_block:", block.verbose_data.is_chain_block)

    @unittest.skip
    async def test_get_subnetwork(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        subnetwork = await rpc.get_subnetwork("0000000000000000000000000000000000000000")
        print("subnetwork.gas:", subnetwork.gas)

    @unittest.skip
    async def test_get_virtual_chain_from_block(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        virtual_chain = await rpc.get_virtual_chain_from_block("f269afa4a29dd7e751e15343d9edbb132e2e2f7daecb71b29908c9f6922ec165", True)
        print("virtual_chain.removed_chain_block_hashes:", virtual_chain.removed_chain_block_hashes)
        print("virtual_chain.added_chain_block_hashes:", virtual_chain.added_chain_block_hashes)
        print("virtual_chain.accepted_transaction_ids[0].accepting_block_hash:", virtual_chain.accepted_transaction_ids[0].accepting_block_hash)
        print("virtual_chain.accepted_transaction_ids[0].accepted_transaction_ids:", virtual_chain.accepted_transaction_ids[0].accepted_transaction_ids)

    @unittest.skip
    async def test_get_blocks(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        blocks = await rpc.get_blocks("f269afa4a29dd7e751e15343d9edbb132e2e2f7daecb71b29908c9f6922ec165", True, True)
        print("blocks.block_hashes", len(blocks.block_hashes))
        print("blocks.blocks", len(blocks.blocks)) # Same as test_get_block

    @unittest.skip
    async def test_get_block_count(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        block_count = await rpc.get_block_count()
        print("block_count.header_count:", block_count.header_count)
        print("block_count.block_count:", block_count.block_count)

    @unittest.skip
    async def test_get_block_dag_info(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        block_dag_info = await rpc.get_block_dag_info()
        print("block_dag_info.network:", block_dag_info.network)
        print("block_dag_info.block_count:", block_dag_info.block_count)
        print("block_dag_info.header_count:", block_dag_info.header_count)
        print("block_dag_info.tip_hashes:", block_dag_info.tip_hashes)
        print("block_dag_info.difficulty:", block_dag_info.difficulty)
        print("block_dag_info.past_median_time:", block_dag_info.past_median_time)
        print("block_dag_info.virtual_parent_hashes:", block_dag_info.virtual_parent_hashes)
        print("block_dag_info.pruning_point_hash:", block_dag_info.pruning_point_hash)
        print("block_dag_info.virtual_daa_score:", block_dag_info.virtual_daa_score)
        print("block_dag_info.sink:", block_dag_info.sink)

    @unittest.skip
    async def test_resolve_finality_conflict(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        await rpc.resolve_finality_conflict("f269afa4a29dd7e751e15343d9edbb132e2e2f7daecb71b29908c9f6922ec165")

    @unittest.skip
    async def test_shutdown(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        await rpc.shutdown()

    @unittest.skip
    async def test_get_headers(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        headers = await rpc.get_headers("f269afa4a29dd7e751e15343d9edbb132e2e2f7daecb71b29908c9f6922ec165", 1000, True)
        print("headers", headers)

    @unittest.skip
    async def test_get_balance_by_address(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        balance = await rpc.get_balance_by_address("pyrin:qzn54t6vpasykvudztupcpwn2gelxf8y9p84szksr73me39mzf69uaalnymtx")
        print("balance:", balance)

    @unittest.skip
    async def test_get_balances_by_addresses(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        balances = await rpc.get_balances_by_addresses(["pyrin:qzn54t6vpasykvudztupcpwn2gelxf8y9p84szksr73me39mzf69uaalnymtx"])
        print("balances.address:", balances[0].address)
        print("balances.balance:", balances[0].balance)

    @unittest.skip
    async def test_get_utxos_by_addresses(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        utxos = await rpc.get_utxos_by_addresses(["pyrin:qzn54t6vpasykvudztupcpwn2gelxf8y9p84szksr73me39mzf69uaalnymtx"])
        print("utxos:", len(utxos))
        print("utxos[0].address", utxos[0].address)
        print("utxos[0].outpoint.transaction_id", utxos[0].outpoint.transaction_id)
        print("utxos[0].outpoint.index", utxos[0].outpoint.index)
        print("utxos[0].utxo_entry.amount", utxos[0].utxo_entry.amount)
        print("utxos[0].utxo_entry.script_public_key", utxos[0].utxo_entry.script_public_key)
        print("utxos[0].utxo_entry.block_daa_score", utxos[0].utxo_entry.block_daa_score)
        print("utxos[0].utxo_entry.is_coinbase", utxos[0].utxo_entry.is_coinbase)

    @unittest.skip
    async def test_get_sink_blue_score(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        sink_blue_score = await rpc.get_sink_blue_score()
        print("sink_blue_score:", sink_blue_score)

    @unittest.skip
    async def test_ban(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        await rpc.ban("192.168.1.2")

    @unittest.skip
    async def test_unban(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        await rpc.unban("192.168.1.2")

    @unittest.skip
    async def test_get_info(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        info = await rpc.get_info()

        print("get_info.p2p_id:", info.p2p_id)
        print("get_info.mempool_size:", info.mempool_size)
        print("get_info.server_version:", info.server_version)
        print("get_info.is_utxo_indexed:", info.is_utxo_indexed)
        print("get_info.is_synced:", info.is_synced)
        print("get_info.has_notify_command:", info.has_notify_command)
        print("get_info.has_message_id:", info.has_message_id)

    @unittest.skip
    async def test_estimate_network_hashes_per_second(self):
        rpc = pyrin.RPC()
        await rpc.connect()
        # window_size: u32, start_hash: Option<RpcHash>
        hashes_per_second = await rpc.estimate_network_hashes_per_second(1000)
        print("hashes_per_second:", hashes_per_second)

    # @unittest.skip
    async def test_notifier(self):
        rpc = pyrin.RPC()
        success = await rpc.connect()

        await rpc.on_block_added(lambda block: print(f"block-added: {block.header.hash}"))
        await rpc.on_finality_conflict(lambda violating_block_hash: print(f"finality-conflict: {violating_block_hash}"))
        await rpc.on_finality_conflict_resolved(lambda finality_block_hash: print(f"finality-conflict-resolved: {finality_block_hash}"))
        await rpc.on_new_block_template(lambda: print(f"new-block-template"))
        await rpc.on_pruning_point_utxo_set_override(lambda _: print(f"pruning-point-utxo-set-override"))
        await rpc.on_sink_blue_score_changed(lambda sink_blue_score: print(f"sink_blue_score: {sink_blue_score}"))
        await rpc.on_virtual_daa_score_changed(lambda score: print(f"virtual-daa-score: {score}"))

        def on_utxos_changed(added, removed):
            print(f"utxos-changed: {added[0].address}, {removed}")

        await rpc.on_utxos_changed(on_utxos_changed, ["pyrin:qqd9p8w75xqe80fx05qs0v3g0fmplztxays0f8x9l0asnn0apanlu9dd79gev"])

        def on_virtual_chain(added_chain_block_hashes, removed_chain_block_hashes, accepted_transaction_ids):
            print(f"virtual-chain: {added_chain_block_hashes}, {removed_chain_block_hashes}, {accepted_transaction_ids}")

        await rpc.on_virtual_chain_changed(on_virtual_chain, True)

        while True:
            await asyncio.sleep(1)

if __name__ == "__main__":
    unittest.main()