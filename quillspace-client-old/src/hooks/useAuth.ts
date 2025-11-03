import { useContext } from '@builder.io/qwik';
import { AuthContextId, type AuthContext } from '~/contexts/auth';

/**
 * Custom hook to access user and tenant information from context
 * Use this anywhere under the tenant routes to get authenticated user and tenant data
 */
export const useAuth = (): AuthContext & { userPath: string; tenantPath: string } => {
  const context = useContext(AuthContextId);
  
  const userPath = context.user && context.tenant 
    ? `/tenants/${context.tenant.slug}/users/${context.user.first_name}`
    : '/';
  
  const tenantPath = context.tenant 
    ? `/tenants/${context.tenant.slug}`
    : '/';
  
  return {
    ...context,
    userPath,
    tenantPath
  };
};
