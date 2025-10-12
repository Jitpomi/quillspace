# useAuth Hook

The `useAuth` hook provides easy access to authenticated user and tenant information throughout the tenant routes.

## Usage

```tsx
import { useAuth } from '~/hooks/useAuth';

export default component$(() => {
  const { user, tenant } = useAuth();
  
  return (
    <div>
      <h1>Welcome, {user?.name}!</h1>
      <p>Tenant: {tenant?.name}</p>
    </div>
  );
});
```

## Available Data

### User Object
- `id`: User ID
- `email`: User email address
- `name`: User display name
- Additional properties from the user_info cookie

### Tenant Object
- `id`: Tenant ID
- `slug`: Tenant URL slug
- `name`: Tenant display name
- Additional properties from the tenant_info cookie

## Context Provider

The auth context is automatically provided by the tenant layout component (`/routes/tenants/[...tenant]/layout.tsx`). All components under the tenant routes have access to this context.

## Implementation Details

- Uses Qwik's `useContext` API
- Data is fetched server-side via `routeLoader$` in the layout
- Context is provided using `useContextProvider` in the layout component
- Automatically available to all child components without prop drilling
