/**
 * Puck Editor Wrapper for Qwik
 * Integrates the Puck visual editor with Qwik using qwikify$
 */

/** @jsxImportSource react */
import { qwikify$ } from "@builder.io/qwik-react";
import { Puck, type Data } from "@measured/puck";
import { puckConfig } from "./puck-config";
import "@measured/puck/puck.css";

interface PuckEditorProps {
  data?: Data;
  onPublish?: (data: Data) => void;
  onChange?: (data: Data) => void;
}

// React component for Puck Editor
const PuckEditorReact = ({ data, onPublish, onChange }: PuckEditorProps) => {
  const initialData: Data = data || {
    content: [
      {
        type: "Hero",
        props: {
          title: "Welcome to My Literary World",
          subtitle: "Discover stories that transport you to new realms",
          ctaText: "Explore My Books",
          ctaLink: "#books",
        },
      },
    ],
    root: { props: { title: "My Author Website" } },
  };

  return (
    <div style={{ height: "100vh" }}>
      <Puck
        config={puckConfig}
        data={initialData}
        onPublish={onPublish}
        onChange={onChange}
      />
    </div>
  );
};

// Qwikified component
export const PuckEditor = qwikify$(PuckEditorReact, {
  eagerness: "hover",
});

export default PuckEditor;
