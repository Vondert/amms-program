export const bigintAbsDiff = (bigint1: bigint, bigint2: bigint) => {
    return bigint1 > bigint2 ? bigint1 - bigint2 : bigint2 - bigint1;
}