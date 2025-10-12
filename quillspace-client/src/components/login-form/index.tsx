import {
    HTMLAttributes, HTMLInputAutocompleteAttribute, HTMLInputTypeAttribute, QwikJSX, Signal, Size,
    component$, $, useSignal, useTask$
} from "@builder.io/qwik";
import {LoginRequest, LoginRequestSchema, LoginResponse} from "~/api/schema";
import {setValue, SubmitHandler, useForm, zodForm$,} from "@modular-forms/qwik";
import {LuEye, LuEyeOff, LuLock, LuMail} from "@qwikest/icons/lucide";
import {globalAction$, useNavigate, zod$} from "@builder.io/qwik-city";
// Qwik Storefront UI components not working properly - keeping original form

export interface LoginFormProps {
    login?: LoginRequest,
}

export type FormFieldProps = {
    pattern?: string | undefined;
    align?: string | undefined;
    name?: string | undefined;
    disabled?: boolean | undefined;
    formAction?: string | undefined;
    formEnctype?: string | undefined;
    formMethod?: string | undefined;
    formNoValidate?: boolean | undefined;
    formTarget?: string | undefined;
    popoverTargetAction?: string | undefined;
    accept?: string | undefined;
    alt?: string | undefined;
    autocomplete?: AutoFill | undefined;
    capture?: string | undefined;
    checked?: boolean | undefined;
    defaultChecked?: boolean | undefined;
    defaultValue?: string | undefined;
    dirName?: string | undefined;
    indeterminate?: boolean | undefined;
    multiple?: boolean | undefined;
    placeholder?: string | undefined;
    readOnly?: boolean | undefined;
    required?: boolean | undefined;
    selectionDirection?: "none" | "forward" | "backward" | null | undefined;
    selectionEnd?: number | null | undefined;
    selectionStart?: number | null | undefined;
    size?: number | undefined;
    src?: string | undefined;
    useMap?: string | undefined;
    valueAsNumber?: number | undefined;
    webkitdirectory?: boolean | undefined;
    autoComplete?: HTMLInputAutocompleteAttribute | Omit<HTMLInputAutocompleteAttribute, string> | undefined;
    'bind:checked'?: Signal<boolean | undefined> | undefined;
    'bind:value'?: Signal<string | number | undefined> | undefined;
    enterKeyHint?: "search" | "enter" | "done" | "go" | "next" | "previous" | "send" | undefined;
    height?: Size | undefined;
    max?: string | number | undefined;
    maxLength?: number | undefined;
    min?: string | number | undefined;
    minLength?: number | undefined;
    step?: string | number | undefined;
    type?: HTMLInputTypeAttribute | undefined;
    value?: number | readonly string[] | FormDataEntryValue | null | undefined;
    width?: Size | undefined;
    children?: undefined;
    popovertarget?: string | undefined;
    popovertargetaction?: ("toggle" | "hide" | "show") | undefined;
}

export const useLogin = globalAction$(async (data, requestEvent) => {
    const {env, cookie} = requestEvent;
    const API_BASE_URL = env.get('VITE_API_BASE_URL');
    
    try {
        const response = await fetch(`${API_BASE_URL}/auth/login`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(data),
        });

        const result: LoginResponse = await response.json();

        if (!response.ok) {
            return {
                success: false,
                error: 'Login failed',
            };
        }
        
        if (result.success) {
            // Store JWT token in secure HTTP-only cookie
            cookie.set('auth_token', result.data.token, {
                httpOnly: true,
                secure: false, // Set to false for localhost development
                sameSite: 'lax',
                maxAge: 60 * 60 * 24 * 7, // 7 days
                path: '/',
            });

            // Store user info in a separate cookie for client-side access
            cookie.set('user_info', JSON.stringify(result.data.user), {
                httpOnly: false, // Accessible to client-side
                secure: false, // Set to false for localhost development
                sameSite: 'lax',
                maxAge: 60 * 60 * 24 * 7,
                path: '/',
            });
            
            cookie.set('tenant_info', JSON.stringify(result.data.tenant), {
                httpOnly: false, // Accessible to client-side
                secure: false, // Set to false for localhost development
                sameSite: 'lax',
                maxAge: 60 * 60 * 24 * 7,
                path: '/',
            });
            
            const tenantSlug = result.data.tenant.slug;
            const firstName = result.data.user.first_name;
            const redirectPath = `/tenants/${tenantSlug}/users/${firstName}`;
            console.log('Attempting redirect to:', redirectPath);
            
            // Return success and let client handle redirect
            return {
                success: true,
                redirectTo: redirectPath,
                error: null,
            };
        }

        return {
            success: false,
            error: 'Invalid response from server',
        };
    } catch (error) {
        // Let redirect responses pass through
        if (error instanceof Response) {
            throw error;
        }

        console.error('Login error:', error);
        return {
            success: false,
            error: 'Network error. Please try again.',
        };
    }
}, zod$(LoginRequestSchema));


/**
 * The RouterHead component is placed inside of the document `<head>` element.
 */
export default  component$<LoginFormProps>((props) => {
    const [loginForm, {Form, Field}] = useForm<LoginRequest>({
        loader: useSignal({email: '', password: ''}),
        validate: zodForm$(LoginRequestSchema)
    });
    const nav = useNavigate();
    const error = useSignal<string | null>(null);
    const showPassword = useSignal(false);
    const togglePasswordVisibility = $(() => {
        showPassword.value = !showPassword.value;
    });
    const loginAction = useLogin();
    const handleSubmit = $<SubmitHandler<LoginRequest>>(async (values) => {
        // Clear any previous errors
        error.value = null;
        
        // Submit the action
        const result = await loginAction.submit(values);
        
        // Handle the result
        if (result.value) {
            if ('error' in result.value && result.value.error) {
                error.value = result.value.error;
            } else if ('redirectTo' in result.value && result.value.redirectTo) {
                // Client-side redirect
                await nav(result.value.redirectTo);
            }
        }
    });

    useTask$(({ track }) => {
        const login = track(() => props.login);
        if (!login) return;
        for (const [key,value] of Object.entries(login)) {
            setValue(loginForm, key as 'email' | 'password', value);
        }
    });

    return (
        <div class="space-y-6">
            <Form onSubmit$={handleSubmit}>
                <div class="space-y-5">
                    <Field name="email">
                        { (
                            field: { value: FormDataEntryValue | null | undefined; error: string },
                            props: QwikJSX.IntrinsicAttributes & FormFieldProps & HTMLAttributes<HTMLInputElement>
                          ) => (
                            <div>
                                {/* Email Field */}
                                <label for="email" class="block text-sm font-medium text-white mb-2">
                                    Email address
                                </label>
                                <div class="relative">
                                    <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                                        <LuMail class="h-5 w-5 text-gray-300"/>
                                    </div>
                                    <input
                                        {...props}
                                        type="email"
                                        class="block w-full pl-12 pr-4 py-4 bg-white/10 backdrop-blur-sm border border-white/20 rounded-xl focus:outline-none focus:ring-0 focus:border-[#9CAF88] focus:bg-white/15 text-white placeholder-gray-300 transition-all duration-300 hover:bg-white/15"
                                        placeholder="Enter your email"
                                        value={field.value}
                                    />
                                </div>
                                {field.error && <div class="mt-2 text-sm text-red-300">{field.error}</div>}
                            </div>
                        )}
                    </Field>
                    
                    <Field name="password">
                        {(
                            field: { value: FormDataEntryValue | null | undefined; error: string },
                            props: QwikJSX.IntrinsicAttributes & FormFieldProps & HTMLAttributes<HTMLInputElement>
                        ) => (
                            <div>
                                <label for="password" class="block text-sm font-medium text-white mb-2">
                                    Password
                                </label>
                                <div class="relative">
                                    <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                                        <LuLock class="h-5 w-5 text-gray-300"/>
                                    </div>
                                    <input
                                        {...props}
                                        type={showPassword.value ? 'text' : 'password'}
                                        class="block w-full pl-12 pr-12 py-4 bg-white/10 backdrop-blur-sm border border-white/20 rounded-xl focus:outline-none focus:ring-0 focus:border-[#9CAF88] focus:bg-white/15 text-white placeholder-gray-300 transition-all duration-300 hover:bg-white/15"
                                        placeholder="Enter your password"
                                        value={field.value}
                                    />
                                    <button
                                        type="button"
                                        onClick$={togglePasswordVisibility}
                                        class="absolute cursor-pointer inset-y-0 right-0 pr-4 flex items-center hover:bg-white/10 rounded-r-xl transition-colors"
                                    >
                                        {showPassword.value ? (
                                            <LuEyeOff class="h-5 w-5 text-gray-300 hover:text-white"/>
                                        ) : (
                                            <LuEye class="h-5 w-5 text-gray-300 hover:text-white"/>
                                        )}
                                    </button>
                                </div>
                                {field.error && <div class="mt-2 text-sm text-red-300">{field.error}</div>}
                            </div>
                        )}
                    </Field>
                </div>
                
                {/* Error Display */}
                {error.value && (
                    <div class="mt-4 p-4 bg-red-500/20 backdrop-blur-sm border border-red-400/30 rounded-xl">
                        <p class="text-sm text-red-200">{error.value}</p>
                    </div>
                )}
                
                <button 
                    type="submit" 
                    class="w-full cursor-pointer mt-8 bg-[#9CAF88] text-white py-4 px-6 rounded-xl hover:bg-[#8ba077] transition-all duration-300 font-semibold text-lg shadow-lg hover:shadow-xl transform hover:scale-[1.02] active:scale-[0.98]"
                >
                    Sign In
                </button>
            </Form>
            
            {/* Demo Credentials */}
            <div class="mt-6 p-4 bg-[#9CAF88]/20 backdrop-blur-sm rounded-xl border border-[#9CAF88]/30">
                <h4 class="text-sm font-medium text-white mb-2">Demo Credentials:</h4>
                <div class="text-xs text-gray-300 space-y-1">
                    <div>Email: yasinkak@gmail.com</div>
                    <div>Password: secret</div>
                </div>
            </div>
        </div>
    );
});
