# User Service API

## Run modes (Postgres/Fluvio vs In-Memory)
- Postgres + Fluvio (default): `cargo run`
- In-memory (DB and events): `cargo run -- --db=memory --events=memory`
- Mix as needed: `cargo run -- --db=postgres --events=memory` (or the opposite)

HTTP service (Axum) to manage users, friendships, and blocks.

## Conventions
- `Authorization: Bearer <jwt>` is required where `Authenticated: Yes` is indicated.
- `from` and `to` parameters are inclusive integers for pagination.
- `:user` in routes represents the id of another user; the authenticated id is taken from the JWT.
- Empty responses indicate success without a payload.

## Swagger-style Endpoints

### GET /health
- Authenticated: No
- Description: Basic service check.
- 200 Response:
```json
{ "status": "ok" }
```

### GET /:user
- Authenticated: No
- Description: Get a user by id.
- 200 Response:
```json
{
  "username": "alice",
  "id": "user-123",
  "created_at": "2024-04-01T12:00:00Z"
}
```

### POST /update
- Authenticated: Yes
- Description: Updates the authenticated user's profile.
- Body (application/json):
```json
{ "username": "new-username" }
```
- 200 Response: Empty.

### GET /:user/is-friend
- Authenticated: Yes
- Description: Checks if the authenticated user is friends with `:user`.
- 200 Response:
```json
true
```

### GET /:user/is-blocked
- Authenticated: Yes
- Description: Checks if the authenticated user blocked `:user`.
- 200 Response:
```json
false
```

### GET /friendship (alias: GET /friendship/friends)
- Authenticated: Yes
- Description: List friends.
- Query params: `from` (int), `to` (int)
- 200 Response:
```json
[
  { "username": "bob", "id": "user-2", "created_at": "2024-03-20T08:15:00Z" }
]
```

### DELETE /friendship/friends/:user
- Authenticated: Yes
- Description: Removes a friendship with `:user`.
- Body: No body.
- 200 Response: Empty.

### POST /friendship/requests/:user
- Authenticated: Yes
- Description: Sends a friend request to `:user`.
- Body: No body.
- 200 Response: Empty.
- Errors:
  - 404: "User does not exist"
  - 409: "Friend request already exists"

### PUT /friendship/requests/:user/accept
- Authenticated: Yes
- Description: Accepts a request received from `:user`.
- Body: No body.
- 200 Response: Empty.
- Errors:
  - 404: "Friend request does not exist" | "User does not exist"
  - 409: "Friend request already handled"

### PUT /friendship/requests/:user/reject
- Authenticated: Yes
- Description: Rejects a request received from `:user`.
- Body: No body.
- 200 Response: Empty.
- Errors:
  - 404: "Friend request does not exist" | "User does not exist"
  - 409: "Friend request already handled"

### DELETE /friendship/requests/:user
- Authenticated: Yes
- Description: Deletes a friend request sent by the authenticated user to `:user`.
- Body: No body.
- 200 Response: Empty.
- Errors:
  - 404: "Friend request does not exist"

### GET /friendship/requests/sent
- Authenticated: Yes
- Description: Sent requests.
- Query params: `from` (int), `to` (int), `filter` (pending|accepted|rejected, optional)
- 200 Response:
```json
[
  {
    "sender_id": "user-1",
    "recipient_id": "user-2",
    "state": "pending",
    "created_at": "2024-03-21T10:00:00Z"
  }
]
```

### GET /friendship/requests/received
- Authenticated: Yes
- Description: Received requests.
- Query params: `from` (int), `to` (int), `filter` (pending|accepted|rejected, optional)
- 200 Response:
```json
[
  {
    "sender_id": "user-3",
    "recipient_id": "user-1",
    "state": "pending",
    "created_at": "2024-03-22T09:30:00Z"
  }
]
```

### GET /blocks
- Authenticated: Yes
- Description: List blocked users.
- Query params: `from` (int), `to` (int)
- 200 Response:
```json
[
  { "sender_id": "user-1", "recipient_id": "user-4", "created_at": "2024-03-15T12:00:00Z" }
]
```

### POST /blocks/:user
- Authenticated: Yes
- Description: Blocks `:user`.
- Body: No body.
- 200 Response: Empty.
- Errors:
  - 404: "User does not exist"
  - 409: "Block already exists"

### DELETE /blocks/:user
- Authenticated: Yes
- Description: Unblocks `:user`.
- Body: No body.
- 200 Response: Empty.
- Errors:
  - 404: "Block does not exist"
