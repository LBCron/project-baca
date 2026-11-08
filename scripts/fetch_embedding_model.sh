#!/usr/bin/env bash
set -euo pipefail

# Downloads the embedding model at a pinned revision into MODEL_DIR, once.
# The compose `embedder-fetch` service runs it before `embedder` starts, so
# the model service itself needs no route out (it sits on `store` only) and
# a restart never depends on huggingface.co. A rerun with the files in
# place downloads nothing; every file the service reads, tokenizer and
# configuration included, is checked against SHA-256.
# text-embeddings-inference serves onnx/model.onnx with ONNX Runtime (about
# twice the passage throughput of its candle backend on CPU) and falls back
# to model.safetensors on a build without it.

MODEL_REPO="${MODEL_REPO:-intfloat/multilingual-e5-base}"
MODEL_REVISION="${MODEL_REVISION:-d128750597153bb5987e10b1c3493a34e5a4502a}"
MODEL_DIR="${MODEL_DIR:-/data/multilingual-e5-base}"
FILES=(config.json tokenizer.json tokenizer_config.json special_tokens_map.json
    sentence_bert_config.json modules.json 1_Pooling/config.json
    model.safetensors onnx/model.onnx)
CHECKSUMS="9dab198f24c8c0879e481cf7822005d5ecbceedbacb390ffafa594e28d31bac4  config.json
62c24cdc13d4c9952d63718d6c9fa4c287974249e16b7ade6d5a85e7bbb75626  tokenizer.json
efb5c0d09722e5fe59a462cd2a9976ee216d55b037597d997cd3fe833216da15  tokenizer_config.json
06e405a36dfe4b9604f484f6a1e619af1a7f7d09e34a8555eb0b77b66318067f  special_tokens_map.json
948201d8329907aae938fa62f9ceeed53f5694dacc2b87b9f3b78b37ee986529  sentence_bert_config.json
c6e29747481e8b5dd2b58401966aeac910de39092f90cda9a704b1545f902b04  modules.json
f586ab6c734af7b8d14d898bfa7faa886f23b46f86dd1730514309893af13d75  1_Pooling/config.json
a18a44fad1d0b46ded15928144138cff1135d5cc8233bdd90be5f18822de09a7  model.safetensors
84a4d426f7e87a6bf5bf195f0bae2c4a7d15f675b23ca96f42fab8326d7a77aa  onnx/model.onnx"

stamp="$MODEL_DIR/.revision"
complete() {
    [[ "$(cat "$stamp" 2>/dev/null)" == "$MODEL_REVISION" ]] || return 1
    for file in "${FILES[@]}"; do [[ -s "$MODEL_DIR/$file" ]] || return 1; done
}
if complete; then
    echo "model $MODEL_REPO@$MODEL_REVISION already in $MODEL_DIR"
    exit 0
fi

mkdir -p "$MODEL_DIR/1_Pooling" "$MODEL_DIR/onnx"
for file in "${FILES[@]}"; do
    echo "fetching $file"
    curl -fsSL --retry 3 -o "$MODEL_DIR/$file.part" \
        "https://huggingface.co/$MODEL_REPO/resolve/$MODEL_REVISION/$file"
    mv "$MODEL_DIR/$file.part" "$MODEL_DIR/$file"
done
(cd "$MODEL_DIR" && echo "$CHECKSUMS" | sha256sum -c -)
echo "$MODEL_REVISION" > "$stamp"
echo "model $MODEL_REPO@$MODEL_REVISION ready in $MODEL_DIR"
