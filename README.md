# User Service API

Servicio HTTP (Axum) para gestionar usuarios, amistades y bloqueos.

## Convenciones
- `Authorization: Bearer <jwt>` requerido donde se indica `Authenticated: Sí`.
- Parámetros `from` y `to` son enteros inclusivos para paginación.
- `user_id` siempre se refiere al otro usuario; el id del autenticado se extrae del JWT.

## Endpoints estilo Swagger

### GET /health
- Authenticated: No
- Descripción: Comprobación básica del servicio.
- Respuesta 200:
```json
"Long life to the allmighty turbofish"
```

### GET /
- Authenticated: No
- Descripción: Obtiene un usuario por id.
- Body (application/json):
```json
{
  "id": "user-123",
  "username": "se-ignora"
}
```
- Respuesta 200:
```json
{
  "username": "alice",
  "id": "user-123",
  "created_at": "2024-04-01T12:00:00Z"
}
```

### POST /update
- Authenticated: Sí
- Descripción: Actualiza el perfil del usuario autenticado.
- Body (application/json):
```json
{ "username": "nuevo-nick" }
```
- Respuesta 200: Vacía.

### GET /friendship
- Authenticated: Sí
- Descripción: Lista de amigos.
- Query params: `from` (int), `to` (int)
- Respuesta 200:
```json
[
  { "username": "bob", "id": "user-2", "created_at": "2024-03-20T08:15:00Z" }
]
```

### POST /friendship/remove
- Authenticated: Sí
- Descripción: Elimina una amistad.
- Body (application/json):
```json
{ "id": "user-2", "username": "opcional" }
```
- Respuesta 200: Vacía.

### POST /friendship/request
- Authenticated: Sí
- Descripción: Envía solicitud de amistad.
- Body (application/json):
```json
{ "user_id": "user-2" }
```
- Respuesta 200: Vacía.
- Errores:
  - 404: "User does not exist"
  - 409: "Friend request already exists"

### POST /friendship/accept
- Authenticated: Sí
- Descripción: Acepta solicitud recibida.
- Body (application/json):
```json
{ "user_id": "user-2" }
```
- Respuesta 200: Vacía.
- Errores:
  - 404: "Friend request does not exist" | "User does not exist"
  - 409: "Friend request already handled"

### POST /friendship/reject
- Authenticated: Sí
- Descripción: Rechaza solicitud recibida.
- Body (application/json):
```json
{ "user_id": "user-2" }
```
- Respuesta 200: Vacía.
- Errores:
  - 404: "Friend request does not exist" | "User does not exist"
  - 409: "Friend request already handled"

### POST /friendship/delete
- Authenticated: Sí
- Descripción: Elimina una solicitud de amistad enviada por el usuario autenticado.
- Body (application/json):
```json
{ "user_id": "user-2" }
```
- Respuesta 200: Vacía.
- Errores:
  - 404: "Friend request does not exist"

### GET /friendship/sent
- Authenticated: Sí
- Descripción: Solicitudes enviadas.
- Query params: `from` (int), `to` (int), `filter` (pending|accepted|rejected, opcional)
- Respuesta 200:
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

### GET /friendship/received
- Authenticated: Sí
- Descripción: Solicitudes recibidas.
- Query params: `from` (int), `to` (int), `filter` (pending|accepted|rejected, opcional)
- Respuesta 200:
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

### POST /friendship/is_friend
- Authenticated: Sí
- Descripción: Comprueba si el usuario autenticado es amigo de otro usuario.
- Body (application/json):
```json
{ "id": "user-2", "username": "opcional" }
```
- Respuesta 200:
```json
true
```

### GET /blocks
- Authenticated: Sí
- Descripción: Lista de usuarios bloqueados.
- Query params: `from` (int), `to` (int)
- Respuesta 200:
```json
[
  { "sender_id": "user-1", "recipient_id": "user-4", "created_at": "2024-03-15T12:00:00Z" }
]
```

### POST /blocks/block
- Authenticated: Sí
- Descripción: Bloquea a un usuario.
- Body (application/json):
```json
{ "user_id": "user-4" }
```
- Respuesta 200: Vacía.
- Errores:
  - 404: "User does not exist"
  - 409: "Block already exists"

### POST /blocks/unblock
- Authenticated: Sí
- Descripción: Desbloquea a un usuario.
- Body (application/json):
```json
{ "user_id": "user-4" }
```
- Respuesta 200: Vacía.
- Errores:
  - 404: "Block does not exist"

### POST /blocks/is_blocked
- Authenticated: Sí
- Descripción: Comprueba si el usuario autenticado bloqueó a otro usuario.
- Body (application/json):
```json
{ "user_id": "user-4" }
```
- Respuesta 200:
```json
true
```
