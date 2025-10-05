/**
 * Actions Context for QuillSpace
 * Provides serializable action handlers following Qwik best practices
 * https://qwik.dev/docs/concepts/think-qwik/
 */

import { 
  component$, 
  createContextId, 
  useContextProvider, 
  useContext, 
  useSignal,
  Slot,
  $,
  type Signal
} from '@builder.io/qwik';

// Action signals interface
export interface ActionSignals {
  // Content actions
  refreshContent: Signal<number>;
  createContent: Signal<number>;
  editContent: Signal<string>;
  deleteContent: Signal<string>;
  publishContent: Signal<string>;
  
  // User actions  
  refreshUsers: Signal<number>;
  createUser: Signal<number>;
  editUser: Signal<string>;
  deleteUser: Signal<string>;
  updateUserRole: Signal<{id: string, role: string}>;
  
  // Tenant actions
  refreshTenants: Signal<number>;
  updateTenantSettings: Signal<any>;
}

// Create context
export const ActionsContext = createContextId<ActionSignals>('actions');

// Provider component
export const ActionsProvider = component$(() => {
  // Content action signals
  const refreshContent = useSignal(0);
  const createContent = useSignal(0);
  const editContent = useSignal('');
  const deleteContent = useSignal('');
  const publishContent = useSignal('');
  
  // User action signals
  const refreshUsers = useSignal(0);
  const createUser = useSignal(0);
  const editUser = useSignal('');
  const deleteUser = useSignal('');
  const updateUserRole = useSignal<{id: string, role: string}>({id: '', role: ''});
  
  // Tenant action signals
  const refreshTenants = useSignal(0);
  const updateTenantSettings = useSignal(null);

  const actions: ActionSignals = {
    refreshContent,
    createContent,
    editContent,
    deleteContent,
    publishContent,
    refreshUsers,
    createUser,
    editUser,
    deleteUser,
    updateUserRole,
    refreshTenants,
    updateTenantSettings,
  };

  useContextProvider(ActionsContext, actions);

  return <Slot />;
});

// Hook to use actions context
export const useActions = () => {
  return useContext(ActionsContext);
};

// Action helper functions (serializable)
export const triggerRefreshContent = $((actions: ActionSignals) => {
  actions.refreshContent.value++;
});

export const triggerCreateContent = $((actions: ActionSignals) => {
  actions.createContent.value++;
});

export const triggerEditContent = $((actions: ActionSignals, id: string) => {
  actions.editContent.value = id;
});

export const triggerDeleteContent = $((actions: ActionSignals, id: string) => {
  actions.deleteContent.value = id;
});

export const triggerPublishContent = $((actions: ActionSignals, id: string) => {
  actions.publishContent.value = id;
});

export const triggerRefreshUsers = $((actions: ActionSignals) => {
  actions.refreshUsers.value++;
});

export const triggerCreateUser = $((actions: ActionSignals) => {
  actions.createUser.value++;
});

export const triggerEditUser = $((actions: ActionSignals, id: string) => {
  actions.editUser.value = id;
});

export const triggerDeleteUser = $((actions: ActionSignals, id: string) => {
  actions.deleteUser.value = id;
});

export const triggerUpdateUserRole = $((actions: ActionSignals, id: string, role: string) => {
  actions.updateUserRole.value = {id, role};
});

export const triggerUpdateTenantSettings = $((actions: ActionSignals, settings: any) => {
  actions.updateTenantSettings.value = settings;
});
