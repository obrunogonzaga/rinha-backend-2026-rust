# Bake the search index into the runtime image

Slice 5 needs to reduce the cost of exact kNN without changing classifier
output. We decided to generate the structured search index at Docker build
time and bake it into the runtime image, alongside the existing reference
artifacts. The dataset is fixed during evaluation, so build-time indexing keeps
startup deterministic, avoids healthcheck risk, and lets every published image
carry a checksumed index for the exact code/data pair it serves.
