import { useState, useEffect } from 'react'
import { STYLES } from '../styles.js'
import * as api from '../api/client.js'

const SHOP_STYLES = `
${STYLES}

.shop-overlay {
  position: fixed; inset: 0; z-index: 100;
  background: rgba(0,0,0,.85);
  display: flex; align-items: center; justify-content: center;
  backdrop-filter: blur(4px);
}

.shop-modal {
  width: 95vw; max-width: 1200px;
  height: 92vh; max-height: 860px;
  background: #0d0e18;
  border: 1px solid #2a2d44;
  border-radius: 4px;
  display: flex; flex-direction: column;
  overflow: hidden;
  box-shadow: 0 0 80px rgba(0,0,0,.9), 0 0 40px rgba(200,150,42,.04);
}

/* ── Header ── */
.shop-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: .7rem 1.2rem;
  background: #0b0c15;
  border-bottom: 1px solid #1e2035;
  flex-shrink: 0;
}

.shop-title-block { display: flex; flex-direction: column; gap: .1rem; }

.shop-title {
  font-family: 'Cinzel', serif; font-size: .85rem;
  letter-spacing: .2em; text-transform: uppercase;
  color: var(--gold);
}

.shop-subtitle {
  font-family: 'Cinzel', serif; font-size: .6rem;
  color: var(--dim); letter-spacing: .12em;
  text-transform: uppercase;
}

.shop-close-btn {
  background: none; border: 1px solid #2a2d44;
  color: var(--dim); font-family: 'Cinzel', serif;
  font-size: .62rem; letter-spacing: .1em;
  padding: .3rem .8rem; border-radius: 2px;
  cursor: pointer; transition: all .15s;
}
.shop-close-btn:hover { border-color: var(--red); color: var(--red); }

/* ── Currency bar ── */
.shop-currency-bar {
  display: flex; align-items: center; gap: 1.5rem;
  padding: .5rem 1.2rem;
  background: #0c0d1a;
  border-bottom: 1px solid #1a1d2e;
  flex-shrink: 0;
}

.currency-label {
  font-family: 'Cinzel', serif; font-size: .55rem;
  letter-spacing: .12em; text-transform: uppercase;
  color: #3a3d55; margin-right: -.8rem;
}

.currency-item {
  display: flex; align-items: center; gap: .3rem;
  font-family: 'Cinzel', serif; font-size: .72rem;
  color: var(--goldl);
}

.currency-icon {
  font-size: .65rem; opacity: .7;
}

.currency-item.pp { color: #c0c8e0; }
.currency-item.gp { color: var(--goldl); }
.currency-item.sp { color: #c0c0c0; }
.currency-item.cp { color: #c8a060; }

/* ── Tabs ── */
.shop-tabs {
  display: flex; gap: 0;
  border-bottom: 1px solid #1a1d2e;
  flex-shrink: 0;
  background: #0b0c15;
}

.shop-tab {
  background: none; border: none;
  font-family: 'Cinzel', serif; font-size: .65rem;
  letter-spacing: .12em; text-transform: uppercase;
  color: var(--dim); padding: .6rem 1.4rem;
  cursor: pointer; transition: all .15s;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
}

.shop-tab:hover { color: var(--goldl); }
.shop-tab.active {
  color: var(--goldl);
  border-bottom-color: var(--gold);
  background: rgba(200,150,42,.04);
}

/* ── Body ── */
.shop-body {
  display: flex; flex: 1; min-height: 0;
}

/* ── Item Grid ── */
.shop-items-panel {
  flex: 1; min-width: 0;
  overflow-y: auto; padding: .75rem;
  scrollbar-width: thin; scrollbar-color: #2a2d44 #0b0c15;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: .6rem;
  align-content: start;
}

.shop-item-card {
  background: #13151f;
  border: 1px solid #1e2035;
  border-radius: 3px;
  padding: .7rem;
  cursor: pointer;
  transition: all .15s;
  display: flex; flex-direction: column; gap: .3rem;
  position: relative;
}

.shop-item-card:hover {
  border-color: rgba(200,150,42,.4);
  background: rgba(200,150,42,.04);
}

.shop-item-card.selected {
  border-color: var(--gold);
  background: rgba(200,150,42,.07);
  box-shadow: 0 0 12px rgba(200,150,42,.15);
}

.shop-item-card.out-of-stock {
  opacity: .4; cursor: not-allowed;
}

.shop-item-card.out-of-stock:hover {
  border-color: #1e2035;
  background: #13151f;
}

.item-card-type {
  font-family: 'Cinzel', serif; font-size: .52rem;
  letter-spacing: .12em; text-transform: uppercase;
  color: #3a3d55; margin-bottom: .1rem;
}

.item-card-name {
  font-family: 'Cinzel', serif; font-size: .72rem;
  color: var(--goldl); letter-spacing: .05em;
  line-height: 1.3;
}

.item-card-desc {
  font-size: .63rem; color: var(--dim);
  line-height: 1.6; flex: 1;
}

.item-card-stats {
  font-size: .6rem; color: #5a5d80;
  font-family: 'Cinzel', serif;
}

.item-card-footer {
  display: flex; align-items: center; justify-content: space-between;
  margin-top: .3rem; padding-top: .35rem;
  border-top: 1px solid #1e2035;
}

.item-price {
  font-family: 'Cinzel', serif; font-size: .68rem;
  color: var(--goldl);
}

.item-qty {
  font-size: .58rem; color: var(--dim);
  font-family: 'Cinzel', serif;
}

.item-qty.low { color: var(--amber); }
.item-qty.out { color: var(--red); }

.rarity-dot {
  position: absolute; top: .5rem; right: .5rem;
  width: 6px; height: 6px; border-radius: 50%;
}
.rarity-dot.common    { background: #4a4d65; }
.rarity-dot.uncommon  { background: #2d6b3a; }
.rarity-dot.rare      { background: #2a4a8a; }
.rarity-dot.very_rare { background: #6a2a8a; }
.rarity-dot.legendary { background: #8a5a10; }

/* ── Inventory panel (sell tab) ── */
.shop-inventory-panel {
  flex: 1; min-width: 0;
  overflow-y: auto; padding: .75rem;
  scrollbar-width: thin; scrollbar-color: #2a2d44 #0b0c15;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: .6rem;
  align-content: start;
}

.inventory-item-card {
  background: #13151f; border: 1px solid #1e2035;
  border-radius: 3px; padding: .7rem;
  cursor: pointer; transition: all .15s;
  display: flex; flex-direction: column; gap: .3rem;
}

.inventory-item-card:hover {
  border-color: rgba(80,160,100,.4);
  background: rgba(80,160,100,.04);
}

.inventory-item-card.selected {
  border-color: var(--grn);
  background: rgba(80,160,100,.07);
}

.sell-price {
  font-family: 'Cinzel', serif; font-size: .65rem;
  color: var(--grn);
}

/* ── Detail panel ── */
.shop-detail-panel {
  width: 260px; flex-shrink: 0;
  background: #0b0c15;
  border-left: 1px solid #1a1d2e;
  display: flex; flex-direction: column;
  overflow: hidden;
}

.detail-header {
  padding: .8rem 1rem;
  border-bottom: 1px solid #1a1d2e;
  flex-shrink: 0;
}

.detail-name {
  font-family: 'Cinzel', serif; font-size: .8rem;
  color: var(--goldl); letter-spacing: .06em;
  line-height: 1.4; margin-bottom: .3rem;
}

.detail-type {
  font-family: 'Cinzel', serif; font-size: .55rem;
  letter-spacing: .14em; text-transform: uppercase;
  color: #3a3d55;
}

.detail-body {
  flex: 1; overflow-y: auto; padding: .8rem 1rem;
  scrollbar-width: thin; scrollbar-color: #2a2d44 #0b0c15;
}

.detail-desc {
  font-size: .68rem; color: var(--dim);
  line-height: 1.8; margin-bottom: .8rem;
}

.detail-stat-row {
  display: flex; justify-content: space-between;
  align-items: center; padding: .2rem 0;
  border-bottom: 1px solid #13151f;
}

.detail-stat-label {
  font-family: 'Cinzel', serif; font-size: .58rem;
  color: #3a3d55; letter-spacing: .08em;
}

.detail-stat-value {
  font-family: 'Cinzel', serif; font-size: .62rem;
  color: var(--text);
}

.detail-price-block {
  margin-top: .8rem; padding: .6rem;
  background: #13151f; border-radius: 2px;
  border: 1px solid #1e2035;
}

.detail-price-label {
  font-family: 'Cinzel', serif; font-size: .55rem;
  letter-spacing: .12em; color: #3a3d55;
  margin-bottom: .3rem;
}

.detail-price-value {
  font-family: 'Cinzel', serif; font-size: .9rem;
  color: var(--goldl);
}

.detail-footer {
  padding: .75rem 1rem;
  border-top: 1px solid #1a1d2e;
  flex-shrink: 0; display: flex; flex-direction: column; gap: .4rem;
}

.qty-row {
  display: flex; align-items: center; gap: .5rem;
}

.qty-label {
  font-family: 'Cinzel', serif; font-size: .58rem;
  color: #3a3d55; letter-spacing: .08em; flex: 1;
}

.qty-btn {
  background: #13151f; border: 1px solid #2a2d44;
  color: var(--dim); width: 24px; height: 24px;
  display: flex; align-items: center; justify-content: center;
  border-radius: 2px; cursor: pointer; font-size: .8rem;
  transition: all .15s;
}
.qty-btn:hover { border-color: var(--gold); color: var(--goldl); }

.qty-value {
  font-family: 'Cinzel', serif; font-size: .72rem;
  color: var(--text); min-width: 20px; text-align: center;
}

.buy-btn {
  background: linear-gradient(135deg, #2a1f06, #4a3510);
  border: 1px solid var(--gold); color: var(--goldl);
  font-family: 'Cinzel', serif; font-size: .65rem;
  letter-spacing: .12em; padding: .5rem;
  border-radius: 2px; cursor: pointer; transition: all .15s;
  text-align: center;
}
.buy-btn:hover { background: linear-gradient(135deg, #4a3510, #6a4e18); }
.buy-btn:disabled { opacity: .3; cursor: not-allowed; }

.sell-btn {
  background: linear-gradient(135deg, #062a10, #104a20);
  border: 1px solid var(--grn); color: #60c080;
  font-family: 'Cinzel', serif; font-size: .65rem;
  letter-spacing: .12em; padding: .5rem;
  border-radius: 2px; cursor: pointer; transition: all .15s;
  text-align: center;
}
.sell-btn:hover { background: linear-gradient(135deg, #104a20, #1a6a30); }
.sell-btn:disabled { opacity: .3; cursor: not-allowed; }

.detail-empty {
  display: flex; align-items: center; justify-content: center;
  flex: 1; padding: 1rem;
  font-family: 'Cinzel', serif; font-size: .65rem;
  color: #2a2d44; text-align: center; line-height: 1.8;
  letter-spacing: .06em;
}

/* ── Feedback toast ── */
.shop-toast {
  position: absolute; bottom: 5rem; left: 50%;
  transform: translateX(-50%);
  background: #0f1020; border: 1px solid var(--gold);
  border-radius: 3px; padding: .4rem 1rem;
  font-family: 'Cinzel', serif; font-size: .65rem;
  color: var(--goldl); letter-spacing: .08em;
  animation: toast-in .2s ease;
  pointer-events: none; white-space: nowrap;
  z-index: 10;
}

.shop-toast.error { border-color: var(--red); color: #ff8080; }
.shop-toast.success { border-color: var(--grn); color: #60c080; }

@keyframes toast-in {
  from { opacity: 0; transform: translateX(-50%) translateY(8px); }
  to   { opacity: 1; transform: translateX(-50%) translateY(0); }
}

.no-items {
  grid-column: 1 / -1;
  display: flex; align-items: center; justify-content: center;
  padding: 3rem;
  font-family: 'Cinzel', serif; font-size: .7rem;
  color: #2a2d44; letter-spacing: .1em;
}
`

// ─── Helpers ──────────────────────────────────────────────────────────────────

function formatPrice(item) {
  const parts = []
  if (item.price_pp > 0) parts.push(`${item.price_pp}pp`)
  if (item.price_gp > 0) parts.push(`${item.price_gp}gp`)
  if (item.price_sp > 0) parts.push(`${item.price_sp}sp`)
  if (item.price_cp > 0) parts.push(`${item.price_cp}cp`)
  return parts.length > 0 ? parts.join(' ') : 'Free'
}

function formatPlayerPrice(item) {
  const parts = []
  if (item.price_pp > 0) parts.push(`${item.price_pp}pp`)
  if (item.price_gp > 0) parts.push(`${item.price_gp}gp`)
  if (item.price_sp > 0) parts.push(`${item.price_sp}sp`)
  if (item.price_cp > 0) parts.push(`${item.price_cp}cp`)
  return parts.join(' ') || 'Free'
}

function sellValue(item) {
  const vals = { common: 1, uncommon: 25, rare: 250, very_rare: 2500, legendary: 25000 }
  return Math.max(1, Math.floor((vals[item.rarity] || 1) / 2))
}

function itemTypeLabel(type) {
  const map = { weapon: 'Weapon', armor: 'Armor', shield: 'Shield',
    consumable: 'Consumable', wondrous: 'Wondrous Item', quest: 'Quest Item' }
  return map[type] || type
}

function itemStats(item) {
  const stats = []
  if (item.damage_die)   stats.push({ label: 'Damage', value: `${item.damage_die} ${item.damage_type || ''}` })
  if (item.weapon_range) stats.push({ label: 'Range', value: item.weapon_range })
  if (item.base_ac)      stats.push({ label: 'AC', value: item.base_ac })
  if (item.armor_type)   stats.push({ label: 'Armor Type', value: item.armor_type })
  if (item.weapon_type)  stats.push({ label: 'Type', value: item.weapon_type })
  return stats
}

function totalCostCp(item, qty) {
  return ((item.price_pp * 1000) + (item.price_gp * 100) + (item.price_sp * 10) + item.price_cp) * qty
}

function playerTotalCp(player) {
  return (player.platinum * 1000) + (player.gold * 100) + (player.silver * 10) + player.copper
}

// ─── Main Component ───────────────────────────────────────────────────────────

export default function ShopModal({
  campaignId,
  player,
  items: playerItems,
  onShopClose,
  onPlayerUpdate,
}) {
  const [tab, setTab] = useState('buy')
  const [shopData, setShopData] = useState(null)
  const [selectedItem, setSelectedItem] = useState(null)
  const [selectedPlayerItem, setSelectedPlayerItem] = useState(null)
  const [quantity, setQuantity] = useState(1)
  const [loading, setLoading] = useState(false)
  const [toast, setToast] = useState(null) // {text, type}
  const [purchases, setPurchases] = useState([])
  const [sales, setSales] = useState([])

  useEffect(() => {
    api.getShopState(campaignId).then(data => {
      if (data) setShopData(data)
    }).catch(() => {})
  }, [campaignId])

  useEffect(() => {
    setQuantity(1)
  }, [selectedItem])

  const showToast = (text, type = 'success') => {
    setToast({ text, type })
    setTimeout(() => setToast(null), 2200)
  }

  const session = shopData?.session
  const shopItems = shopData?.items || []

  // ── Buy ────────────────────────────────────────────────────────────────────

  const handleBuy = async () => {
    if (!selectedItem || loading) return
    const cost = totalCostCp(selectedItem, quantity)
    if (cost > playerTotalCp(player)) {
      showToast('Insufficient funds', 'error'); return
    }
    setLoading(true)
    try {
      const result = await api.buyItem(campaignId, selectedItem.id, quantity)
      if (result.error) {
        showToast(result.error, 'error')
      } else {
        showToast(`Purchased ${quantity > 1 ? `${quantity}x ` : ''}${selectedItem.name}`, 'success')
        setPurchases(p => [...p, { name: selectedItem.name, qty: quantity }])
        // Refresh shop stock
        const fresh = await api.getShopState(campaignId)
        if (fresh) setShopData(fresh)
        setSelectedItem(null)
        setQuantity(1)
        if (onPlayerUpdate) await onPlayerUpdate()
      }
    } catch (e) {
      showToast('Purchase failed', 'error')
    }
    setLoading(false)
  }

  // ── Sell ───────────────────────────────────────────────────────────────────

  const handleSell = async () => {
    if (!selectedPlayerItem || loading) return
    setLoading(true)
    try {
      const result = await api.sellItem(campaignId, selectedPlayerItem.id)
      if (result.error) {
        showToast(result.error, 'error')
      } else {
        showToast(`Sold ${selectedPlayerItem.name} for ${result.gold_received}gp`, 'success')
        setSales(s => [...s, { name: selectedPlayerItem.name }])
        setSelectedPlayerItem(null)
        if (onPlayerUpdate) await onPlayerUpdate()
      }
    } catch (e) {
      showToast('Sale failed', 'error')
    }
    setLoading(false)
  }

  // ── Close ──────────────────────────────────────────────────────────────────

  const handleClose = async () => {
    try {
      await api.closeShop(campaignId)
    } catch (e) {}
    const purchaseList = purchases.map(p => `${p.qty > 1 ? `${p.qty}x ` : ''}${p.name}`).join(', ')
    const saleList = sales.map(s => s.name).join(', ')
    onShopClose(purchaseList, saleList)
  }

  // ── Inventory items available to sell (not equipped quest items) ───────────
  const sellableItems = (playerItems || []).filter(i =>
    !i.is_equipped && i.item_type !== 'quest'
  )

  const canAfford = selectedItem
    ? totalCostCp(selectedItem, quantity) <= playerTotalCp(player)
    : false

  const availableQty = selectedItem
    ? selectedItem.quantity - selectedItem.quantity_sold
    : 0

  // ─────────────────────────────────────────────────────────────────────────
  return (
    <>
      <style dangerouslySetInnerHTML={{ __html: SHOP_STYLES }} />
      <div className="shop-overlay">
        <div className="shop-modal" style={{ position: 'relative' }}>

          {/* Header */}
          <div className="shop-header">
            <div className="shop-title-block">
              <div className="shop-title">
                {session?.shop_name || 'Shop'}
              </div>
              <div className="shop-subtitle">
                {session?.shop_type?.replace('_', ' ') || 'General Store'}
              </div>
            </div>
            <button className="shop-close-btn" onClick={handleClose}>
              Leave Shop
            </button>
          </div>

          {/* Currency Bar */}
          <div className="shop-currency-bar">
            <div className="currency-label">Your Purse</div>
            {player.platinum > 0 && (
              <div className="currency-item pp">
                <span className="currency-icon">◆</span>
                {player.platinum}pp
              </div>
            )}
            <div className="currency-item gp">
              <span className="currency-icon">◈</span>
              {player.gold}gp
            </div>
            {player.silver > 0 && (
              <div className="currency-item sp">
                <span className="currency-icon">◇</span>
                {player.silver}sp
              </div>
            )}
            {player.copper > 0 && (
              <div className="currency-item cp">
                <span className="currency-icon">○</span>
                {player.copper}cp
              </div>
            )}
          </div>

          {/* Tabs */}
          <div className="shop-tabs">
            <button
              className={`shop-tab${tab === 'buy' ? ' active' : ''}`}
              onClick={() => { setTab('buy'); setSelectedItem(null); setSelectedPlayerItem(null) }}
            >
              Browse
            </button>
            <button
              className={`shop-tab${tab === 'sell' ? ' active' : ''}`}
              onClick={() => { setTab('sell'); setSelectedItem(null); setSelectedPlayerItem(null) }}
            >
              Sell
            </button>
          </div>

          {/* Body */}
          <div className="shop-body">

            {/* ── Buy Tab ── */}
            {tab === 'buy' && (
              <div className="shop-items-panel">
                {shopItems.length === 0 && (
                  <div className="no-items">No items available</div>
                )}
                {shopItems.map(item => {
                  const qty = item.quantity - item.quantity_sold
                  const outOfStock = qty <= 0
                  return (
                    <div
                      key={item.id}
                      className={[
                        'shop-item-card',
                        selectedItem?.id === item.id ? 'selected' : '',
                        outOfStock ? 'out-of-stock' : ''
                      ].filter(Boolean).join(' ')}
                      onClick={() => !outOfStock && setSelectedItem(item)}
                    >
                      <div className={`rarity-dot ${item.rarity}`} />
                      <div className="item-card-type">{itemTypeLabel(item.item_type)}</div>
                      <div className="item-card-name">{item.name}</div>
                      <div className="item-card-desc">{item.description}</div>
                      {item.damage_die && (
                        <div className="item-card-stats">
                          {item.damage_die} {item.damage_type}
                        </div>
                      )}
                      {item.base_ac && (
                        <div className="item-card-stats">AC {item.base_ac}</div>
                      )}
                      <div className="item-card-footer">
                        <div className="item-price">{formatPrice(item)}</div>
                        <div className={`item-qty${qty <= 2 ? ' low' : ''}${outOfStock ? ' out' : ''}`}>
                          {outOfStock ? 'Sold Out' : `${qty} left`}
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            )}

            {/* ── Sell Tab ── */}
            {tab === 'sell' && (
              <div className="shop-inventory-panel">
                {sellableItems.length === 0 && (
                  <div className="no-items">Nothing to sell</div>
                )}
                {sellableItems.map(item => (
                  <div
                    key={item.id}
                    className={[
                      'inventory-item-card',
                      selectedPlayerItem?.id === item.id ? 'selected' : ''
                    ].filter(Boolean).join(' ')}
                    onClick={() => setSelectedPlayerItem(item)}
                  >
                    <div className="item-card-type">{itemTypeLabel(item.item_type)}</div>
                    <div className="item-card-name">{item.name}</div>
                    <div className="item-card-desc">{item.description}</div>
                    <div className="item-card-footer">
                      <div className="sell-price">{sellValue(item)}gp</div>
                      <div className="item-qty">{item.rarity}</div>
                    </div>
                  </div>
                ))}
              </div>
            )}

            {/* ── Detail Panel ── */}
            <div className="shop-detail-panel">
              {tab === 'buy' && !selectedItem && (
                <div className="detail-empty">
                  Select an item<br />to see details
                </div>
              )}
              {tab === 'sell' && !selectedPlayerItem && (
                <div className="detail-empty">
                  Select an item<br />from your inventory<br />to sell
                </div>
              )}

              {/* Buy detail */}
              {tab === 'buy' && selectedItem && (
                <>
                  <div className="detail-header">
                    <div className="detail-name">{selectedItem.name}</div>
                    <div className="detail-type">
                      {itemTypeLabel(selectedItem.item_type)} · {selectedItem.rarity}
                    </div>
                  </div>
                  <div className="detail-body">
                    <div className="detail-desc">{selectedItem.description}</div>
                    {itemStats(selectedItem).map(s => (
                      <div key={s.label} className="detail-stat-row">
                        <span className="detail-stat-label">{s.label}</span>
                        <span className="detail-stat-value">{s.value}</span>
                      </div>
                    ))}
                    {selectedItem.notes && (
                      <div className="detail-desc" style={{ marginTop: '.6rem', color: '#5a5d80' }}>
                        {selectedItem.notes}
                      </div>
                    )}
                    <div className="detail-price-block">
                      <div className="detail-price-label">Price per item</div>
                      <div className="detail-price-value">{formatPrice(selectedItem)}</div>
                    </div>
                    {quantity > 1 && (
                      <div className="detail-price-block" style={{ marginTop: '.4rem', borderColor: canAfford ? '#2a4a20' : '#4a2020' }}>
                        <div className="detail-price-label">Total</div>
                        <div className="detail-price-value" style={{ color: canAfford ? 'var(--goldl)' : 'var(--red)' }}>
                          {selectedItem.price_pp * quantity > 0 ? `${selectedItem.price_pp * quantity}pp ` : ''}
                          {selectedItem.price_gp * quantity > 0 ? `${selectedItem.price_gp * quantity}gp ` : ''}
                          {selectedItem.price_sp * quantity > 0 ? `${selectedItem.price_sp * quantity}sp ` : ''}
                          {selectedItem.price_cp * quantity > 0 ? `${selectedItem.price_cp * quantity}cp` : ''}
                        </div>
                      </div>
                    )}
                  </div>
                  <div className="detail-footer">
                    {availableQty > 1 && (
                      <div className="qty-row">
                        <span className="qty-label">Quantity</span>
                        <button className="qty-btn"
                          onClick={() => setQuantity(q => Math.max(1, q - 1))}>−</button>
                        <span className="qty-value">{quantity}</span>
                        <button className="qty-btn"
                          onClick={() => setQuantity(q => Math.min(availableQty, q + 1))}>+</button>
                      </div>
                    )}
                    <button
                      className="buy-btn"
                      disabled={!canAfford || loading}
                      onClick={handleBuy}
                    >
                      {!canAfford ? 'Cannot Afford' : loading ? 'Purchasing…' : `Purchase${quantity > 1 ? ` ×${quantity}` : ''}`}
                    </button>
                  </div>
                </>
              )}

              {/* Sell detail */}
              {tab === 'sell' && selectedPlayerItem && (
                <>
                  <div className="detail-header">
                    <div className="detail-name">{selectedPlayerItem.name}</div>
                    <div className="detail-type">
                      {itemTypeLabel(selectedPlayerItem.item_type)} · {selectedPlayerItem.rarity}
                    </div>
                  </div>
                  <div className="detail-body">
                    <div className="detail-desc">{selectedPlayerItem.description}</div>
                    <div className="detail-price-block" style={{ borderColor: '#1a3a2a' }}>
                      <div className="detail-price-label">Merchant offers</div>
                      <div className="detail-price-value" style={{ color: '#60c080' }}>
                        {sellValue(selectedPlayerItem)}gp
                      </div>
                    </div>
                    <div className="detail-desc" style={{ marginTop: '.5rem', color: '#3a3d55', fontSize: '.6rem' }}>
                      Merchants pay half the standard value for used goods.
                    </div>
                  </div>
                  <div className="detail-footer">
                    <button
                      className="sell-btn"
                      disabled={loading}
                      onClick={handleSell}
                    >
                      {loading ? 'Selling…' : `Sell for ${sellValue(selectedPlayerItem)}gp`}
                    </button>
                  </div>
                </>
              )}
            </div>
          </div>

          {/* Toast */}
          {toast && (
            <div className={`shop-toast ${toast.type}`}>{toast.text}</div>
          )}

        </div>
      </div>
    </>
  )
}