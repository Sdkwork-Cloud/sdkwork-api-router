# Admin Portal Separation Design

## Goal

Turn the current combined browser console into two independent applications:

- an `admin` operator console with its own login, session, and password change flow
- a `portal` end-user app with its own login, session, and password change flow

Both applications must remain quick-start friendly on the default SQLite setup and ship with local-development default users.

## Current Problems

- The browser console is a single hash-routed shell that mixes `admin` and `portal`.
- The admin API does not have a real password-based account system. It issues JWTs from an arbitrary `subject`.
- The portal API supports registration and login, but it does not expose password change or a default seeded user.
- Frontend app routes and backend API paths both use `/admin` and `/portal`, which prevents clean app separation.

## Recommended Approach

### 1. Split the frontend into two independent apps

Use a Vite multi-page setup with separate entry points:

- `/admin/`
- `/portal/`
- `/` as a lightweight landing page

Move browser-to-backend proxy traffic onto frontend-only API prefixes:

- `/api/admin` -> backend `/admin`
- `/api/portal` -> backend `/portal`
- `/api/v1` -> backend `/v1`

This keeps app URLs and API URLs independent and avoids route collisions.

### 2. Add a real admin account model

Create an `admin_users` persistence model with password hashing, profile loading, login, and password change. Seed a default local admin user automatically when the store is empty for that default account.

Default local admin credentials:

- email: `admin@sdkwork.local`
- password: `ChangeMe123!`

### 3. Seed a default portal user

Keep portal as a separate user boundary and seed a local-development default portal user if it does not exist.

Default local portal credentials:

- email: `portal@example.com`
- password: `ChangeMe123!`

### 4. Add password change endpoints for both apps

Add authenticated password change routes:

- `POST /admin/auth/change-password`
- `POST /portal/auth/change-password`

Both routes must verify the current password and persist the new password hash.

## Backend Shape

### Admin API

- `POST /admin/auth/login` accepts `email` and `password`
- `GET /admin/auth/me` returns the authenticated admin profile
- `POST /admin/auth/change-password` updates the current admin password

### Portal API

- keep `register`, `login`, `me`, `workspace`, and `api-keys`
- add `POST /portal/auth/change-password`
- seed a default portal user on demand before login

## Frontend Shape

### Admin App

- own entry point and document
- own local storage token key
- own login form
- own dashboard shell
- own password change form

### Portal App

- own entry point and document
- own local storage token key
- own login/register/dashboard flow
- own password change form
- no admin navigation embedded in the app

## Testing Strategy

- backend auth tests for default admin login, invalid password rejection, `me`, and password change
- backend portal tests for seeded default portal login and password change
- integration tests for admin route guards still requiring valid bearer tokens
- frontend verification with `pnpm --dir console typecheck`
- runtime verification by starting the workspace and checking:
  - `/admin/`
  - `/portal/`
  - `/api/admin/auth/login`
  - `/api/portal/auth/login`

## Tradeoffs

- This introduces a second persisted user model for admin instead of reusing portal users. That is intentional. Operators and portal end users are different security domains.
- The frontend now uses API proxy prefixes under `/api/*`, but the backend public API remains unchanged.
