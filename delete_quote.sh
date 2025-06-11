#!/bin/bash

# Helper function to delete a quote by ID
delete_quote() {
  local id="$1"

  curl -s -X DELETE "http://localhost:8000/api/v1/delete-quote/$id"
  echo -e "\n🗑️  Deleted quote ID: $id"
}

# Delete quotes with IDs 200 to 210
for id in {200..400}; do
  delete_quote "$id"
done

