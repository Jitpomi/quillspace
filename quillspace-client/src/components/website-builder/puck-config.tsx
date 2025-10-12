/**
 * Puck Editor Configuration for QuillSpace
 * Defines components available in the visual editor for author (website-builder)
 */

/** @jsxImportSource react */
import type { Config } from "@measured/puck";

// Component Props Types
export interface HeroProps {
  title: string;
  subtitle: string;
  backgroundImage?: string;
  ctaText: string;
  ctaLink: string;
}

export interface BookShowcaseProps {
  title: string;
  books: Array<{
    id: string;
    title: string;
    cover: string;
    description: string;
    buyLink: string;
    price: string;
  }>;
}

export interface AuthorBioProps {
  name: string;
  bio: string;
  photo: string;
  socialLinks: Array<{
    platform: string;
    url: string;
  }>;
}

export interface TestimonialProps {
  testimonials: Array<{
    id: string;
    quote: string;
    author: string;
    source?: string;
  }>;
}

export interface NewsletterSignupProps {
  title: string;
  description: string;
  placeholder: string;
  buttonText: string;
  provider: 'mailchimp' | 'convertkit' | 'custom';
  apiEndpoint?: string;
}

export interface BlogPostsProps {
  title: string;
  postsToShow: number;
  showExcerpts: boolean;
  source: 'internal' | 'external';
  feedUrl?: string;
}

// Puck Configuration
export const puckConfig: Config = {
  components: {
    Hero: {
      fields: {
        title: { type: "text" },
        subtitle: { type: "textarea" },
        backgroundImage: { type: "text" },
        ctaText: { type: "text" },
        ctaLink: { type: "text" },
      },
      defaultProps: {
        title: "Welcome to My Literary World",
        subtitle: "Discover stories that transport you to new realms",
        ctaText: "Explore My Books",
        ctaLink: "#books",
      },
      render: ({ title, subtitle, backgroundImage, ctaText, ctaLink }: HeroProps) => (
        <section 
          className="relative h-screen flex items-center justify-center text-white"
          style={{
            backgroundImage: backgroundImage ? `url(${backgroundImage})` : 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
            backgroundSize: 'cover',
            backgroundPosition: 'center'
          }}
        >
          <div className="absolute inset-0 bg-black bg-opacity-40"></div>
          <div className="relative z-10 text-center max-w-4xl px-6">
            <h1 className="text-5xl md:text-7xl font-bold mb-6">{title}</h1>
            <p className="text-xl md:text-2xl mb-8 opacity-90">{subtitle}</p>
            <a 
              href={ctaLink}
              className="inline-block bg-white text-gray-900 px-8 py-4 rounded-lg font-semibold text-lg hover:bg-gray-100 transition-colors"
            >
              {ctaText}
            </a>
          </div>
        </section>
      ),
    },

    BookShowcase: {
      fields: {
        title: { type: "text" },
        books: {
          type: "array",
          arrayFields: {
            id: { type: "text" },
            title: { type: "text" },
            cover: { type: "text" },
            description: { type: "textarea" },
            buyLink: { type: "text" },
            price: { type: "text" },
          },
          getItemSummary: (item: any) => item.title || "New Book",
        },
      },
      defaultProps: {
        title: "My Books",
        books: [
          {
            id: "1",
            title: "The Whispering Pages",
            cover: "/book-covers/placeholder.jpg",
            description: "A haunting tale of mystery and magic...",
            buyLink: "#",
            price: "$12.99",
          },
        ],
      },
      render: ({ title, books }: BookShowcaseProps) => (
        <section className="py-16 bg-gray-50">
          <div className="max-w-6xl mx-auto px-6">
            <h2 className="text-4xl font-bold text-center mb-12 text-gray-900">{title}</h2>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
              {books.map((book) => (
                <div key={book.id} className="bg-white rounded-lg shadow-lg overflow-hidden">
                  <img 
                    src={book.cover} 
                    alt={book.title}
                    className="w-full h-64 object-cover"
                  />
                  <div className="p-6">
                    <h3 className="text-xl font-semibold mb-2">{book.title}</h3>
                    <p className="text-gray-600 mb-4">{book.description}</p>
                    <div className="flex justify-between items-center">
                      <span className="text-2xl font-bold text-green-600">{book.price}</span>
                      <a 
                        href={book.buyLink}
                        className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 transition-colors"
                      >
                        Buy Now
                      </a>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </section>
      ),
    },

    AuthorBio: {
      fields: {
        name: { type: "text" },
        bio: { type: "textarea" },
        photo: { type: "text" },
        socialLinks: {
          type: "array",
          arrayFields: {
            platform: { type: "text" },
            url: { type: "text" },
          },
          getItemSummary: (item: any) => item.platform || "Social Link",
        },
      },
      defaultProps: {
        name: "Your Name",
        bio: "Tell your readers about yourself, your writing journey, and what inspires you...",
        photo: "/author-photos/placeholder.jpg",
        socialLinks: [
          { platform: "Twitter", url: "#" },
          { platform: "Instagram", url: "#" },
          { platform: "Goodreads", url: "#" },
        ],
      },
      render: ({ name, bio, photo, socialLinks }: AuthorBioProps) => (
        <section className="py-16 bg-white">
          <div className="max-w-4xl mx-auto px-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-12 items-center">
              <div>
                <img 
                  src={photo} 
                  alt={name}
                  className="w-full max-w-md mx-auto rounded-lg shadow-lg"
                />
              </div>
              <div>
                <h2 className="text-4xl font-bold mb-6 text-gray-900">About {name}</h2>
                <p className="text-lg text-gray-700 mb-8 leading-relaxed">{bio}</p>
                <div className="flex space-x-4">
                  {socialLinks.map((link, index) => (
                    <a 
                      key={index}
                      href={link.url}
                      className="text-blue-600 hover:text-blue-800 font-medium"
                    >
                      {link.platform}
                    </a>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </section>
      ),
    },

    Testimonials: {
      fields: {
        testimonials: {
          type: "array",
          arrayFields: {
            id: { type: "text" },
            quote: { type: "textarea" },
            author: { type: "text" },
            source: { type: "text" },
          },
          getItemSummary: (item: any) => item.author || "New Testimonial",
        },
      },
      defaultProps: {
        testimonials: [
          {
            id: "1",
            quote: "A masterpiece that kept me turning pages all night long.",
            author: "Sarah Johnson",
            source: "Goodreads Review",
          },
        ],
      },
      render: ({ testimonials }: TestimonialProps) => (
        <section className="py-16 bg-gray-100">
          <div className="max-w-6xl mx-auto px-6">
            <h2 className="text-4xl font-bold text-center mb-12 text-gray-900">What Readers Say</h2>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
              {testimonials.map((testimonial) => (
                <div key={testimonial.id} className="bg-white p-6 rounded-lg shadow-md">
                  <blockquote className="text-gray-700 mb-4 italic">
                    "{testimonial.quote}"
                  </blockquote>
                  <div className="text-sm">
                    <div className="font-semibold text-gray-900">{testimonial.author}</div>
                    {testimonial.source && (
                      <div className="text-gray-500">{testimonial.source}</div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </section>
      ),
    },

    NewsletterSignup: {
      fields: {
        title: { type: "text" },
        description: { type: "textarea" },
        placeholder: { type: "text" },
        buttonText: { type: "text" },
        provider: {
          type: "select",
          options: [
            { label: "Mailchimp", value: "mailchimp" },
            { label: "ConvertKit", value: "convertkit" },
            { label: "Custom", value: "custom" },
          ],
        },
        apiEndpoint: { type: "text" },
      },
      defaultProps: {
        title: "Stay Connected",
        description: "Get updates on new releases, exclusive content, and behind-the-scenes insights.",
        placeholder: "Enter your email address",
        buttonText: "Subscribe",
        provider: "mailchimp" as const,
      },
      render: ({ title, description, placeholder, buttonText }: NewsletterSignupProps) => (
        <section className="py-16 bg-blue-600 text-white">
          <div className="max-w-4xl mx-auto px-6 text-center">
            <h2 className="text-4xl font-bold mb-4">{title}</h2>
            <p className="text-xl mb-8 opacity-90">{description}</p>
            <form className="max-w-md mx-auto flex gap-4">
              <input 
                type="email"
                placeholder={placeholder}
                className="flex-1 px-4 py-3 rounded-lg text-gray-900"
              />
              <button 
                type="submit"
                className="bg-white text-blue-600 px-6 py-3 rounded-lg font-semibold hover:bg-gray-100 transition-colors"
              >
                {buttonText}
              </button>
            </form>
          </div>
        </section>
      ),
    },

    BlogPosts: {
      fields: {
        title: { type: "text" },
        postsToShow: { type: "number" },
        showExcerpts: { type: "radio", options: [
          { label: "Yes", value: true },
          { label: "No", value: false },
        ]},
        source: {
          type: "select",
          options: [
            { label: "Internal Blog", value: "internal" },
            { label: "External RSS Feed", value: "external" },
          ],
        },
        feedUrl: { type: "text" },
      },
      defaultProps: {
        title: "Latest Posts",
        postsToShow: 3,
        showExcerpts: true,
        source: "internal" as const,
      },
      render: ({ title, postsToShow, showExcerpts }: BlogPostsProps) => (
        <section className="py-16 bg-white">
          <div className="max-w-6xl mx-auto px-6">
            <h2 className="text-4xl font-bold text-center mb-12 text-gray-900">{title}</h2>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
              {Array.from({ length: postsToShow }).map((_, index) => (
                <article key={index} className="bg-gray-50 rounded-lg overflow-hidden shadow-md">
                  <div className="h-48 bg-gray-300"></div>
                  <div className="p-6">
                    <h3 className="text-xl font-semibold mb-2">Sample Blog Post {index + 1}</h3>
                    {showExcerpts && (
                      <p className="text-gray-600 mb-4">
                        This is a sample excerpt from a blog post. It gives readers a preview of what to expect...
                      </p>
                    )}
                    <a href="#" className="text-blue-600 hover:text-blue-800 font-medium">
                      Read More →
                    </a>
                  </div>
                </article>
              ))}
            </div>
          </div>
        </section>
      ),
    },
  },
};

export default puckConfig;
