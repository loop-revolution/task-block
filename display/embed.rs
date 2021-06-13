use crate::blocks::task_block::TaskBlock;
use block_tools::{
	auth::{
		optional_token, optional_validate_token,
		permissions::{has_perm_level, PermLevel},
	},
	blocks::Context,
	display_api::{
		component::{
			atomic::{
				badge::BadgeComponent,
				icon::{Icon, IconComponent},
				text::TextComponent,
			},
			form::dropdown::DropdownComponent,
			interact::{
				button::{ButtonComponent, ButtonVariant},
				link::LinkComponent,
			},
			layout::{
				card::{CardComponent, DetachedMenu},
				stack::{SpacingOptions, StackComponent},
			},
			menus::menu::MenuComponent,
			DisplayComponent,
		},
		ActionObject, RedirectObject,
	},
	models::Block,
	LoopError,
};

impl TaskBlock {
	pub fn handle_embed_display(
		block: &Block,
		context: &Context,
	) -> Result<DisplayComponent, LoopError> {
		let conn = &context.conn()?;
		let user_id = optional_validate_token(optional_token(context))?;

		let Self {
			name,
			description,
			status,
			deps,
		} = Self::from_id(block.id, user_id, conn)?;

		let mut icon_col = StackComponent::vertical();
		let mut content_col = StackComponent::vertical();

		let mut first_row = StackComponent {
			spacing: Some(SpacingOptions::Between),
			..StackComponent::fit()
		};

		let mut name_blocked_stack = StackComponent::fit();

		let name = name
			.and_then(|block| block.block_data)
			.unwrap_or_else(|| "Untitled Task".into());
		let text = TextComponent {
			bold: Some(true),
			..TextComponent::new(name)
		};
		let link = LinkComponent {
			app_path: Some(format!("/b/{}", block.id)),
			no_style: Some(true),
			..LinkComponent::new(text)
		};
		name_blocked_stack.push(link);

		// Dependency logic
		let mut blocked_by = vec![];
		for dep in deps {
			let Self { status, name, .. } = Self::from_id(dep.id, user_id, conn)?;
			// If it's not done, add to deps list
			if status.and_then(|status| status.block_data) != Some("2".to_string()) {
				let name = name
					.and_then(|name| name.block_data)
					.unwrap_or_else(|| "Untitled Task".to_string());
				let redirect = RedirectObject::app_path(format!("b/{}", dep.id));
				let action = ActionObject::redirect(redirect);
				let button = ButtonComponent {
					icon: Some(Icon::TaskComplete),
					interact: Some(action),
					variant: Some(ButtonVariant::Outline),
					..ButtonComponent::new(name)
				};
				blocked_by.push(button);
			}
		}
		if !blocked_by.is_empty() {
			let blocked = BadgeComponent::new("Blocked");
			name_blocked_stack.push(blocked);
		}

		first_row.push(name_blocked_stack);
		let mut status_dropdown = DropdownComponent {
			disabled: Some(true),
			..Self::status(status, block.id)
		};

		let mut detached_menu = None;

		if let Some(user_id) = user_id {
			if has_perm_level(user_id, block, PermLevel::Edit) {
				status_dropdown.disabled = Some(false)
			}
			let mut menu = MenuComponent::from_block(block, user_id);
			menu.load_comments(conn)?;
			detached_menu = Some(DetachedMenu::bottom_right(menu));
		}
		first_row.push(status_dropdown);
		content_col.push(first_row);

		if let Some(desc) = Self::description(&description, false) {
			content_col.push(desc)
		}

		if !blocked_by.is_empty() {
			let mut blocked_row = StackComponent::fit();
			blocked_row.push(TextComponent::info("Blocked by"));
			for button in blocked_by {
				blocked_row.push(button)
			}
			content_col.push(blocked_row);
		}

		icon_col.push(IconComponent::new(Icon::TaskComplete));

		let mut content = StackComponent::horizontal();
		content.push(icon_col);
		content.push(content_col);

		Ok(CardComponent {
			color: block.color.clone(),
			detached_menu,
			..CardComponent::new(content)
		}
		.into())
	}
}
