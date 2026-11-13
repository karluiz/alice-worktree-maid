const demo = document.querySelector('.tray-demo')
const trigger = document.querySelector('[data-tray-trigger]')
const menu = document.querySelector('[data-tray-menu]')
const status = document.querySelector('[data-preview-status]')
const focusables = [...menu.querySelectorAll('[data-menu-focus]')]
const appearanceButtons = [...document.querySelectorAll('[data-appearance-choice]')]
const variantButtons = [...document.querySelectorAll('[data-variant-choice]')]

function focusFirstAction() {
  focusables[0]?.focus({ preventScroll: true })
}

function openMenu({ focus = false } = {}) {
  menu.hidden = false
  trigger.setAttribute('aria-expanded', 'true')
  trigger.setAttribute('aria-label', 'Close Alice tray menu')
  status.textContent = 'Menu open · Alice is ready'
  if (focus) focusFirstAction()
}

function closeMenu({ restoreFocus = false } = {}) {
  menu.hidden = true
  trigger.setAttribute('aria-expanded', 'false')
  trigger.setAttribute('aria-label', 'Open Alice tray menu')
  if (restoreFocus) trigger.focus({ preventScroll: true })
}

function updateImages() {
  const monochrome = demo.dataset.appearance === 'monochrome'
  const source = monochrome ? 'assets/alice-mono.png' : demo.dataset.variant === 'pixel' ? 'assets/alice-pixel.png' : 'assets/alice-line.png'
  for (const image of demo.querySelectorAll('[data-alice-image]')) {
    image.src = source
    image.width = source.includes('alice-pixel') ? 236 : 265
    image.height = source.includes('alice-pixel') ? 256 : 288
  }
}

function setAppearance(appearance) {
  demo.dataset.appearance = appearance
  for (const button of appearanceButtons) {
    button.setAttribute('aria-pressed', String(button.dataset.appearanceChoice === appearance))
  }
  updateImages()
  status.textContent = appearance === 'monochrome' ? 'Monochrome appearance selected' : 'Color appearance selected'
}

function setVariant(variant) {
  demo.dataset.variant = variant === 'mono' ? 'line' : variant
  setAppearance(variant === 'mono' ? 'monochrome' : 'color')
  for (const button of variantButtons) {
    const selected = button.dataset.variantChoice === variant
    button.setAttribute('aria-pressed', String(selected))
    button.querySelector('.icon-option__choice').textContent = selected ? 'Selected' : 'Choose treatment'
  }
  updateImages()
  status.textContent = `${variant === 'pixel' ? 'Pixel' : variant === 'mono' ? 'Monochrome' : 'Line'} treatment selected`
}

trigger.addEventListener('click', () => {
  if (trigger.getAttribute('aria-expanded') === 'true') closeMenu()
  else openMenu()
})

trigger.addEventListener('keydown', event => {
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    openMenu({ focus: true })
  }
})

menu.addEventListener('keydown', event => {
  const current = focusables.indexOf(document.activeElement)
  if (event.key === 'Escape') {
    event.preventDefault()
    closeMenu({ restoreFocus: true })
  } else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    const direction = event.key === 'ArrowDown' ? 1 : -1
    const next = (current + direction + focusables.length) % focusables.length
    focusables[next]?.focus()
  } else if (event.key === 'Home') {
    event.preventDefault()
    focusables[0]?.focus()
  } else if (event.key === 'End') {
    event.preventDefault()
    focusables.at(-1)?.focus()
  }
})

document.addEventListener('pointerdown', event => {
  if (trigger.getAttribute('aria-expanded') !== 'true') return
  if (!menu.contains(event.target) && !trigger.contains(event.target)) closeMenu()
})

document.addEventListener('keydown', event => {
  if (event.key === 'Escape' && trigger.getAttribute('aria-expanded') === 'true') closeMenu({ restoreFocus: true })
})

for (const link of document.querySelectorAll('[data-open-menu]')) {
  link.addEventListener('click', event => {
    event.preventDefault()
    document.querySelector('#top').scrollIntoView()
    openMenu({ focus: true })
  })
}

for (const button of menu.querySelectorAll('[data-action]')) {
  button.addEventListener('click', () => {
    const action = button.dataset.action
    if (action === 'Close') {
      closeMenu({ restoreFocus: true })
      return
    }
    status.textContent = `${action} selected · preview only`
  })
}

for (const button of appearanceButtons) {
  button.addEventListener('click', () => setAppearance(button.dataset.appearanceChoice))
}

for (const button of variantButtons) {
  button.addEventListener('click', () => setVariant(button.dataset.variantChoice))
}
