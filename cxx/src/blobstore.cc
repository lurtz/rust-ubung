#include "cxx-experiment/include/blobstore.h"
#include "cxx-experiment/src/main.rs.h"
#include <algorithm>
#include <functional>
#include <set>
#include <string>
#include <unordered_map>

namespace org {
namespace blobstore {

// Toy implementation of an in-memory blobstore.
//
// In reality the implementation of BlobstoreClient could be a large complex C++
// library.
class BlobstoreClient_impl : public BlobstoreClient {
public:
  BlobstoreClient_impl() = default;

  // Upload a new blob and return a blobid that serves as a handle to the blob.
  uint64_t put(MultiBuf &buf) override {
    std::string contents;

    // Traverse the caller's chunk iterator.
    //
    // In reality there might be sophisticated batching of chunks and/or
    // parallel upload implemented by the blobstore's C++ client.
    while (true) {
      auto chunk = next_chunk(buf);
      if (chunk.size() == 0) {
        break;
      }
      contents.append(reinterpret_cast<const char *>(chunk.data()),
                      chunk.size());
    }

    // Insert into map and provide caller the handle.
    auto blobid = std::hash<std::string>{}(contents);
    blobs[blobid] = {std::move(contents), {}};
    return blobid;
  }

  // Add tag to an existing blob.
  void tag(uint64_t blobid, rust::Str tag) override {
    blobs[blobid].tags.emplace(tag);
  }

  // Retrieve metadata about a blob.
  BlobMetadata metadata(uint64_t blobid) override {
    BlobMetadata metadata{};
    auto blob = blobs.find(blobid);
    if (blob != blobs.end()) {
      metadata.size = blob->second.data.size();
      std::for_each(blob->second.tags.cbegin(), blob->second.tags.cend(),
                    [&](auto &t) { metadata.tags.emplace_back(t); });
    }
    return metadata;
  }

private:
  using Blob = struct {
    std::string data;
    std::set<std::string> tags;
  };
  std::unordered_map<uint64_t, Blob> blobs;
};

std::unique_ptr<BlobstoreClient> new_blobstore_client() {
  return std::make_unique<BlobstoreClient_impl>();
}

std::unique_ptr<Int_wrapper> create_int_wrapper(std::uint8_t const val) {
  return std::make_unique<Int_wrapper>(val);
}

} // namespace blobstore
} // namespace org
