import { component$ } from '@builder.io/qwik';
import {routeLoader$} from "@builder.io/qwik-city";
import {getAuthToken, getTenantInfo, getUserInfo} from "~/utils/auth";

export const useConnectedWebsite = routeLoader$(async (requestEventAction) => {
  const { cookie, params } = requestEventAction;
  const user = await getUserInfo(cookie);
  const tenant = await getTenantInfo(cookie);
  const token = getAuthToken(cookie);
  if (!user || !tenant) {
    throw new Error('User not authenticated');
  }

  try {
    // Call the backend API to get connected websites
    const response = await fetch(`${process.env.BACKEND_URL || 'http://localhost:3001'}/api/connected-websites/websites/${params.website}`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch websites: ${response.statusText}`);
    }

    const website = await response.json();
    return { success: true, website };
  } catch (error) {
    console.error('Error fetching connected websites:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    };
  }
});

export default component$(() => {
  const website = useConnectedWebsite();
  
  return (
    <>
    Website: {JSON.stringify(website.value)}
    </>
  );
});

