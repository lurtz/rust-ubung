#pragma once
#include "rust/cxx.h"
#include <memory>

namespace org {
namespace blobstore {

struct MultiBuf;
struct BlobMetadata;

class BlobstoreClient {
public:
  BlobstoreClient() = default;
  virtual ~BlobstoreClient() = default;
  virtual uint64_t put(MultiBuf &buf) = 0;
  virtual void tag(uint64_t blobid, rust::Str tag) = 0;
  virtual BlobMetadata metadata(uint64_t blobid) = 0;
};

std::unique_ptr<BlobstoreClient> new_blobstore_client();

} // namespace blobstore
} // namespace org
