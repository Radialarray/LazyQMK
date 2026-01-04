import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Button from '$components/Button.svelte';

describe('Button Component', () => {
	it('renders with default props', () => {
		const { container } = render(Button, {
			props: {
				children: () => 'Click me'
			}
		});
		const button = container.querySelector('button');
		expect(button).toBeTruthy();
		expect(button?.textContent).toContain('Click me');
	});

	it('applies variant classes correctly', () => {
		const { container } = render(Button, {
			props: {
				variant: 'destructive',
				children: () => 'Delete'
			}
		});
		const button = container.querySelector('button');
		expect(button?.className).toContain('destructive');
	});

	it('handles disabled state', () => {
		const { container } = render(Button, {
			props: {
				disabled: true,
				children: () => 'Disabled'
			}
		});
		const button = container.querySelector('button');
		expect(button?.disabled).toBe(true);
	});

	it('handles click events', async () => {
		let clicked = false;
		const { container } = render(Button, {
			props: {
				onclick: () => {
					clicked = true;
				},
				children: () => 'Click'
			}
		});
		const button = container.querySelector('button');
		button?.click();
		expect(clicked).toBe(true);
	});
});
