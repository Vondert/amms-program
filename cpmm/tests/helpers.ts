import {
    airdropFactory,
    generateKeyPairSigner, KeyPairSigner,
    lamports,
    Rpc,
    RpcSubscriptions,
    SolanaRpcApi,
    SolanaRpcSubscriptionsApi
} from "@solana/web3.js";
import {Program} from "@coral-xyz/anchor";
import { Cpmm } from "../target/types/cpmm";

const LAMPORTS_PER_SOL = BigInt(1_000_000_000);
export type CpmmTestingEnvironment = {
    program: Program<Cpmm>;
    rpc: Rpc<SolanaRpcApi>;
    rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;
};
export const getTestUser = async (rpc: Rpc<SolanaRpcApi>, rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>, airdrop_amount: number): Promise<KeyPairSigner> =>{
    const user = await generateKeyPairSigner();
    console.log("Generated user address:", user.address);

    const airdrop = airdropFactory({ rpc, rpcSubscriptions });
    await airdrop({
        commitment: 'processed',
        lamports: lamports(LAMPORTS_PER_SOL * BigInt(airdrop_amount)),
        recipientAddress: user.address
    });

    console.log(`Airdrop of ${airdrop_amount} SOL completed for user:`, user.address);

    return user;
}