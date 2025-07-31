# Spec-to-Proof UI

A Next.js 14 frontend for the Spec-to-Proof platform, featuring an invariant disambiguation wizard with comprehensive editing capabilities.

## Features

### 🎯 Invariant Disambiguation Wizard
- **Review & Edit**: Modify invariant descriptions, formal expressions, and metadata
- **Split Invariants**: Break down complex invariants into simpler components
- **Rename**: Quick renaming of invariant descriptions
- **Bulk Operations**: Select multiple invariants for batch processing
- **Status Management**: Confirm, reject, or restore invariants

### 🎨 Modern UI/UX
- **Responsive Design**: Works seamlessly on desktop and mobile
- **Accessibility**: WCAG 2.1 AA compliant with proper ARIA labels
- **Performance**: Optimized for Lighthouse score ≥ 95
- **Theme Support**: Light/dark mode with system preference detection

### 🔧 Technical Features
- **Type Safety**: Full TypeScript with Zod validation
- **State Management**: tRPC with React Query for efficient data fetching
- **Form Validation**: Client-side validation with error handling
- **Real-time Updates**: Optimistic updates with error recovery

## Architecture

### Core Components
- `InvariantCard`: Main invariant display with action buttons
- `InvariantList`: List view with filtering and pagination
- `EditInvariantModal`: Full-featured editing interface
- `SplitInvariantModal`: Complex invariant splitting tool
- `RenameInvariantModal`: Quick rename functionality

### Data Flow
1. **tRPC Client**: Type-safe API calls with automatic caching
2. **React Query**: Optimistic updates and error handling
3. **Zod Validation**: End-to-end type safety
4. **Toast Notifications**: User feedback for all actions

### Performance Optimizations
- **Code Splitting**: Automatic route-based code splitting
- **Image Optimization**: Next.js Image component with lazy loading
- **Bundle Analysis**: Webpack optimizations for production builds
- **Caching**: React Query with configurable stale times

## Getting Started

### Prerequisites
- Node.js 20+
- npm 9+

### Installation
```bash
cd platform/ui
npm install
```

### Development
```bash
npm run dev
```

The application will be available at `http://localhost:3000`

### Building for Production
```bash
npm run build
npm start
```

## Testing

### Unit Tests
```bash
npm test
```

### E2E Tests (Cypress)
```bash
npm run cypress:open
```

### Accessibility Tests
```bash
npm run test:a11y
```

## Performance Monitoring

### Lighthouse Score
The application is optimized to achieve a Lighthouse score ≥ 95:

- **Performance**: 95+
- **Accessibility**: 100
- **Best Practices**: 95+
- **SEO**: 100

### Key Optimizations
- **Bundle Splitting**: Vendor and app code separated
- **Tree Shaking**: Unused code eliminated
- **Image Optimization**: WebP format with responsive sizes
- **Font Loading**: Optimized font loading with `display: swap`

## Accessibility Features

### WCAG 2.1 AA Compliance
- **Keyboard Navigation**: Full keyboard accessibility
- **Screen Reader Support**: Proper ARIA labels and roles
- **Color Contrast**: Meets AA standards (4.5:1 ratio)
- **Focus Management**: Visible focus indicators
- **Reduced Motion**: Respects user motion preferences

### Semantic HTML
- Proper heading hierarchy
- Landmark regions (header, main, nav)
- Form labels and descriptions
- Error announcements

## Component Library

### Core UI Components
- `Button`: Accessible button with variants
- `Input`: Form input with validation
- `Modal`: Accessible modal dialogs
- `Toast`: Notification system
- `Badge`: Status and priority indicators

### Layout Components
- `Header`: Application navigation
- `Card`: Content containers
- `LoadingSpinner`: Loading states

### Form Components
- `Textarea`: Multi-line text input
- `Select`: Dropdown selection
- `TagInput`: Tag management
- `ConfirmDialog`: Confirmation dialogs

## API Integration

### tRPC Setup
The application uses tRPC for type-safe API communication:

```typescript
// Example tRPC query
const { data, isLoading, error } = trpc.invariant.list.useQuery({
  documentId: '123',
  status: 'EXTRACTED',
  limit: 20,
  offset: 0,
});

// Example tRPC mutation
const updateInvariant = trpc.invariant.update.useMutation({
  onSuccess: () => {
    toast.success('Invariant updated successfully');
    refetch();
  },
  onError: (error) => {
    toast.error(`Failed to update invariant: ${error.message}`);
  },
});
```

### Error Handling
- **Network Errors**: Automatic retry with exponential backoff
- **Validation Errors**: Client-side Zod validation
- **User Feedback**: Toast notifications for all actions

## Deployment

### Environment Variables
```bash
NEXT_PUBLIC_API_URL=http://localhost:3001/trpc
NEXT_PUBLIC_APP_URL=http://localhost:3000
NEXT_PUBLIC_GOOGLE_VERIFICATION=your-verification-code
```

### Production Build
```bash
npm run build
npm start
```

### Docker Deployment
```dockerfile
FROM node:20-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
RUN npm run build
EXPOSE 3000
CMD ["npm", "start"]
```

## Contributing

### Code Style
- **TypeScript**: Strict mode enabled
- **ESLint**: Airbnb configuration
- **Prettier**: Consistent formatting
- **Husky**: Pre-commit hooks

### Testing Strategy
- **Unit Tests**: Jest with React Testing Library
- **Integration Tests**: API integration testing
- **E2E Tests**: Cypress for critical user flows
- **Accessibility Tests**: axe-core integration

### Performance Budget
- **First Contentful Paint**: < 1.5s
- **Largest Contentful Paint**: < 2.5s
- **Cumulative Layout Shift**: < 0.1
- **First Input Delay**: < 100ms

## Troubleshooting

### Common Issues

#### Build Errors
```bash
# Clear Next.js cache
rm -rf .next
npm run build
```

#### TypeScript Errors
```bash
# Check for type errors
npm run type-check
```

#### Performance Issues
```bash
# Analyze bundle
npm run analyze
```

### Debug Mode
```bash
# Enable debug logging
DEBUG=* npm run dev
```

## License

MIT License - see LICENSE file for details. 