"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Skeleton } from "@/components/ui/skeleton"
import { RefreshCw, TrendingUp, TrendingDown, Activity, Clock, Hash, ArrowUpRight, ArrowDownLeft } from "lucide-react"

interface NetflowData {
  cumulative_net: string
  last_block: number
  updated_at: string
}

interface Transfer {
  tx_hash: string
  block_number: number
  from_address: string
  to_address: string
  amount: string
  direction: "IN" | "OUT"
  timestamp: string
}

const API_BASE = "http://127.0.0.1:8080"
const TOKEN = "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063"

export default function PolygonNetflowDashboard() {
  const [netflow, setNetflow] = useState<NetflowData | null>(null)
  const [transfers, setTransfers] = useState<Transfer[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [lastUpdated, setLastUpdated] = useState<Date>(new Date())

  const loadNetflow = async () => {
    try {
      const res = await fetch(`${API_BASE}/netflow?token=${TOKEN}`)
      if (!res.ok) throw new Error("Failed to fetch netflow data")
      const data = await res.json()
      setNetflow(data)
      setError(null)
    } catch (err) {
      console.error("Netflow fetch error:", err)
      setError("Failed to load netflow data")
    }
  }

  const loadTransfers = async () => {
    try {
      const res = await fetch(`${API_BASE}/transfers?token=${TOKEN}&limit=10`)
      if (!res.ok) throw new Error("Failed to fetch transfers")
      const data = await res.json()
      setTransfers(data)
      setError(null)
    } catch (err) {
      console.error("Transfers fetch error:", err)
      setError("Failed to load transfer data")
    }
  }

  const loadData = async () => {
    setLoading(true)
    await Promise.all([loadNetflow(), loadTransfers()])
    setLoading(false)
    setLastUpdated(new Date())
  }

  useEffect(() => {
    loadData()
    const interval = setInterval(loadData, 15000)
    return () => clearInterval(interval)
  }, [])

  const formatAmount = (amount: string) => {
    const num = Number.parseFloat(amount)
    if (isNaN(num)) return amount
    if (num >= 1e9) return `${(num / 1e9).toFixed(2)}B`
    if (num >= 1e6) return `${(num / 1e6).toFixed(2)}M`
    if (num >= 1e3) return `${(num / 1e3).toFixed(2)}K`
    return num.toFixed(4)
  }

  const formatAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`
  }

  const formatTime = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString()
  }

  const isPositiveFlow = netflow ? Number.parseFloat(netflow.cumulative_net) > 0 : false

  return (
    <div className="min-h-screen bg-background text-foreground p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">Polygon Netflow Dashboard</h1>
            <p className="text-muted-foreground mt-1">Real-time monitoring of POL token transfers</p>
          </div>
          <div className="flex items-center gap-4">
            <div className="text-sm text-muted-foreground">Last updated: {lastUpdated.toLocaleTimeString()}</div>
            <Button onClick={loadData} disabled={loading} size="sm" className="gap-2">
              <RefreshCw className={`h-4 w-4 ${loading ? "animate-spin" : ""}`} />
              Refresh
            </Button>
          </div>
        </div>

        {/* Error Alert */}
        {error && (
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {/* Netflow Metrics */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Cumulative Netflow</CardTitle>
              {isPositiveFlow ? (
                <TrendingUp className="h-4 w-4 text-green-500" />
              ) : (
                <TrendingDown className="h-4 w-4 text-red-500" />
              )}
            </CardHeader>
            <CardContent>
              {loading ? (
                <Skeleton className="h-8 w-32" />
              ) : (
                <div className="text-2xl font-bold">
                  {netflow ? formatAmount(netflow.cumulative_net) : "0"}
                  <span className="text-sm font-normal text-muted-foreground ml-2">POL</span>
                </div>
              )}
              <Badge variant={isPositiveFlow ? "default" : "destructive"} className="mt-2">
                {isPositiveFlow ? "Net Inflow" : "Net Outflow"}
              </Badge>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Last Block</CardTitle>
              <Hash className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              {loading ? (
                <Skeleton className="h-8 w-24" />
              ) : (
                <div className="text-2xl font-bold">{netflow?.last_block?.toLocaleString() || "0"}</div>
              )}
              <p className="text-xs text-muted-foreground mt-2">Latest processed block</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Active Monitoring</CardTitle>
              <Activity className="h-4 w-4 text-green-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-green-500">Live</div>
              <p className="text-xs text-muted-foreground mt-2">Updates every 15 seconds</p>
            </CardContent>
          </Card>
        </div>

        {/* Recent Transfers */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Clock className="h-5 w-5" />
              Recent Transfers
            </CardTitle>
            <CardDescription>Latest 10 POL token transfers on Polygon network</CardDescription>
          </CardHeader>
          <CardContent>
            {loading ? (
              <div className="space-y-3">
                {[...Array(5)].map((_, i) => (
                  <Skeleton key={i} className="h-12 w-full" />
                ))}
              </div>
            ) : (
              <div className="rounded-md border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Transaction</TableHead>
                      <TableHead>Block</TableHead>
                      <TableHead>From</TableHead>
                      <TableHead>To</TableHead>
                      <TableHead className="text-right">Amount</TableHead>
                      <TableHead>Direction</TableHead>
                      <TableHead>Time</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {transfers.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={7} className="text-center text-muted-foreground py-8">
                          No transfers found
                        </TableCell>
                      </TableRow>
                    ) : (
                      transfers.map((transfer, index) => (
                        <TableRow key={index}>
                          <TableCell className="font-mono text-sm">{formatAddress(transfer.tx_hash)}</TableCell>
                          <TableCell>{transfer.block_number.toLocaleString()}</TableCell>
                          <TableCell className="font-mono text-sm">{formatAddress(transfer.from_address)}</TableCell>
                          <TableCell className="font-mono text-sm">{formatAddress(transfer.to_address)}</TableCell>
                          <TableCell className="text-right font-mono">{formatAmount(transfer.amount)}</TableCell>
                          <TableCell>
                            <Badge variant={transfer.direction === "IN" ? "default" : "secondary"} className="gap-1">
                              {transfer.direction === "IN" ? (
                                <ArrowDownLeft className="h-3 w-3" />
                              ) : (
                                <ArrowUpRight className="h-3 w-3" />
                              )}
                              {transfer.direction}
                            </Badge>
                          </TableCell>
                          <TableCell className="text-sm text-muted-foreground">
                            {formatTime(transfer.timestamp)}
                          </TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}