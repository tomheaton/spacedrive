
export type ClientCommand = { key: "FileRead", params: { id: bigint, } } | { key: "FileDelete", params: { id: bigint, } } | { key: "LibDelete", params: { id: bigint, } } | { key: "TagCreate", params: { name: string, color: string, } } | { key: "TagAssign", params: { file_id: bigint, tag_id: bigint, } } | { key: "TagDelete", params: { id: bigint, } } | { key: "LocScan", params: { id: bigint, } } | { key: "LocDelete", params: { id: bigint, } } | { key: "LocUpdate", params: { id: bigint, name: string | null, } } | { key: "SysVolumeUnmount", params: { id: bigint, } };