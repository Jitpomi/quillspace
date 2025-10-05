import { component$, useSignal } from '@builder.io/qwik';
import { LuUsers, LuPlus, LuPencil, LuTrash2, LuMail, LuCalendar } from '@qwikest/icons/lucide';
import { 
  useActions, 
  triggerCreateUser, 
  triggerEditUser, 
  triggerDeleteUser, 
  triggerUpdateUserRole 
} from '../../contexts/actions';

interface User {
  id: string;
  email: string;
  name: string;
  role: 'admin' | 'editor' | 'viewer';
  created_at: string;
  updated_at: string;
  is_active: boolean;
}

interface UserManagementProps {
  users: User[];
  currentUserRole: string;
}

export const UserManagement = component$<UserManagementProps>(({ 
  users, 
  currentUserRole
}) => {
  const actions = useActions();
  const selectedRole = useSignal<string>('all');

  const filteredUsers = users.filter(user => 
    selectedRole.value === 'all' || user.role === selectedRole.value
  );

  const getRoleColor = (role: string) => {
    switch (role) {
      case 'admin': return 'bg-red-100 text-red-800';
      case 'editor': return 'bg-blue-100 text-blue-800';
      case 'viewer': return 'bg-gray-100 text-gray-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const canManageUsers = currentUserRole === 'admin';

  return (
    <div class="space-y-6">
      {/* Header */}
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <LuUsers class="w-8 h-8 text-purple-600" />
          <h2 class="text-3xl font-bold text-gray-900">User Management</h2>
        </div>
        {canManageUsers && (
          <button
            onClick$={() => triggerCreateUser(actions)}
            class="bg-purple-600 hover:bg-purple-700 text-white px-4 py-2 rounded-lg font-semibold transition-colors flex items-center gap-2"
          >
            <LuPlus class="w-4 h-4" />
            New User
          </button>
        )}
      </div>

      {/* Filters */}
      <div class="bg-white rounded-xl shadow-lg p-6">
        <div class="flex gap-2 mb-4">
          {['all', 'admin', 'editor', 'viewer'].map((role) => (
            <button
              key={role}
              onClick$={() => selectedRole.value = role}
              class={`px-4 py-2 rounded-lg font-medium transition-colors ${
                selectedRole.value === role
                  ? 'bg-purple-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              {role.charAt(0).toUpperCase() + role.slice(1)}
            </button>
          ))}
        </div>

        {/* User List */}
        <div class="space-y-4">
          {filteredUsers.map((user) => (
            <div key={user.id} class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow">
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-3 mb-2">
                    <h3 class="text-lg font-semibold text-gray-900">{user.name}</h3>
                    <span class={`px-2 py-1 rounded-full text-xs font-medium ${getRoleColor(user.role)}`}>
                      {user.role}
                    </span>
                    {!user.is_active && (
                      <span class="px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-800">
                        Inactive
                      </span>
                    )}
                  </div>
                  <div class="flex items-center gap-4 text-sm text-gray-600 mb-3">
                    <div class="flex items-center gap-1">
                      <LuMail class="w-4 h-4" />
                      {user.email}
                    </div>
                  </div>
                  <div class="flex items-center gap-4 text-xs text-gray-500">
                    <div class="flex items-center gap-1">
                      <LuCalendar class="w-3 h-3" />
                      Joined {new Date(user.created_at).toLocaleDateString()}
                    </div>
                    <div class="flex items-center gap-1">
                      <LuCalendar class="w-3 h-3" />
                      Updated {new Date(user.updated_at).toLocaleDateString()}
                    </div>
                  </div>
                </div>
                {canManageUsers && (
                  <div class="flex items-center gap-2">
                    <select
                      onChange$={(e) => triggerUpdateUserRole(actions, user.id, (e.target as HTMLSelectElement).value)}
                      value={user.role}
                      class="border border-gray-300 rounded px-2 py-1 text-sm"
                    >
                      <option value="viewer">Viewer</option>
                      <option value="editor">Editor</option>
                      <option value="admin">Admin</option>
                    </select>
                    <button
                      onClick$={() => triggerEditUser(actions, user.id)}
                      class="bg-blue-600 hover:bg-blue-700 text-white p-2 rounded transition-colors"
                    >
                      <LuPencil class="w-4 h-4" />
                    </button>
                    <button
                      onClick$={() => triggerDeleteUser(actions, user.id)}
                      class="bg-red-600 hover:bg-red-700 text-white p-2 rounded transition-colors"
                    >
                      <LuTrash2 class="w-4 h-4" />
                    </button>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>

        {filteredUsers.length === 0 && (
          <div class="text-center py-12">
            <LuUsers class="w-12 h-12 text-gray-400 mx-auto mb-4" />
            <p class="text-gray-600">No users found</p>
            {canManageUsers && (
              <button
                onClick$={() => triggerCreateUser(actions)}
                class="mt-4 bg-purple-600 hover:bg-purple-700 text-white px-4 py-2 rounded-lg font-semibold transition-colors"
              >
                Invite your first user
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
});
