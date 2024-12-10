#pragma once
#include "rust/cxx.h"
#include <cstdint>
#include <memory>
#include <utility>

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

template <typename T> struct T_wrapper {
  T t;
  T_wrapper(T t) : t{std::move(t)} {}
  T two_times() const { return t + t; }
};

using Int_wrapper = T_wrapper<uint8_t>;

std::unique_ptr<Int_wrapper> create_int_wrapper(std::uint8_t val);

} // namespace blobstore
} // namespace org
